use handlebars::Handlebars;
use handlebars_misc_helpers::register;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, VecDeque, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{path::Path};
use tauri::State;

use crate::utils::{extract_code_block, ResultTrait};
use crate::{utils::resolve_path, AppState, Problem};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub author: String,
    pub code: Code,
    pub include: HashMap<String, String>,
    pub editor: String,
    pub toggle: ToggleSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToggleSettings {
    pub create_file: bool,
    pub run_on_save: bool,
    pub submit_on_ac: bool,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Code {
    pub filename: String,
    pub template: String,
    pub modifier: String,
    pub lib_check_regex: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            author: "GOD".into(),
            code: Code {
                filename: r#"
{{#with (regex_captures
  pattern="problemset/problem/(\\d+)/([A-Za-z0-9]+)"
  on=url) as |url_parts|}}
  {{#with (regex_captures
    pattern="^[A-Za-z0-9]+\\.\\s*(.+)$"
    on=../title) as |title_parts|}}
./src/bin/{{url_parts._1}}-{{to_lower_case url_parts._2}}-{{to_kebab_case title_parts._1}}.rs
  {{/with}}
{{/with}}
"#
                .into(),
                template: "".into(),
                modifier: r#"
{{!-- Triple braces to prevent html escaping --}}
{{!-- Base code block --}}
{{{code}}}

{{!-- Iterate over each library in lib_files --}}
{{#each lib_files}}
mod {{@key}} {
    {{{this}}}
}
{{/each}}
"#
                .into(),
                lib_check_regex: "use.*{{name}}(::|;)".into(),
            },
            include: HashMap::new(),
            editor: "code".into(),
            toggle: ToggleSettings {
                create_file: true,
                run_on_save: true,
                submit_on_ac: false,
            },
        }
    }
}

#[derive(Serialize)]
struct TemplateData {
    code: String,
    lib_files: Vec<(String, String)>,
}

impl Config {
    pub fn get_filename(&self, problem: &Problem) -> Result<String, String> {
        let mut bars = Handlebars::new();
        register(&mut bars);
        bars.register_template_string("filename", &self.code.filename)
            .map_to_string()?;
        let mut data = BTreeMap::new();
        data.insert("title", problem.title.clone());
        data.insert("url", problem.url.clone());
        let name = bars.render("filename", &data).map_to_string()?;
        let name = name.trim().to_string();
        println!("name = {name}");
        Ok(name)
    }

    pub fn get_file_path(&self, problem: &Problem, dir: &Path) -> Result<PathBuf, String> {
        Ok(resolve_path(dir, &self.get_filename(problem)?))
    }

    fn get_included_files(&self, dir: &Path) -> Result<HashMap<String, String>, String> {
        self.include
            .clone()
            .into_iter()
            .map(|(key, value)| {
                let path = resolve_path(dir, &value);
                let mut files: HashMap<String, PathBuf> = HashMap::new();
                if path.is_dir() {
                    for f in path.read_dir().map_to_string()? {
                        if let Ok(file) = f {
                            if let Some(file_name) = file.path().file_stem() {
                                files.insert(file_name.to_string_lossy().to_string(), file.path());
                            }
                        }
                    }
                } else {
                    files.insert(key, path);
                }
                files
                    .into_iter()
                    .filter(|(_, v)| v.is_file())
                    .map(|(k, v)| match fs::read_to_string(&v) {
                        Ok(content) => Ok((k, content)),
                        Err(e) => Err(format!("Failed to read file {:?}: {}", v, e)),
                    })
                    .collect::<Result<Vec<_>, String>>()
            })
            .collect::<Result<Vec<_>, String>>()
            .map(|v| v.concat().into_iter().collect())
    }

    pub fn get_template(&self, dir: &Path) -> String {
        let template_path = resolve_path(dir, &self.code.template);
        fs::read_to_string(template_path).unwrap_or_else(|e| {
            eprintln!("Error reading template file: {}", e);
            String::new()
        })
    }

    pub fn get_final_code(&self, problem: &Problem, dir: &Path) -> Result<String, String> {
        // Read source code
        let source_code = fs::read_to_string(self.get_file_path(problem, dir)?).map_to_string()?;

        // Get included files content
        let included_files = self.get_included_files(dir)?;

        let mut deque = VecDeque::new();
        let mut visited = HashMap::new();
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

        let mut bars = Handlebars::new();
        register(&mut bars);
        bars.register_template_string("libcheck", &self.code.lib_check_regex)
            .map_to_string()?;

        // Determine initial dependencies from source_code
        for (k, _) in &included_files {
            let re = Regex::new(
                &bars
                    .render("libcheck", &json!({"name": k}))
                    .map_to_string()?,
            )
                .map_to_string_mess("Invalid regex for lib_check")?;
            if re.is_match(&source_code) {
                deque.push_back(k.clone());
            }
        }

        println!("inital libraries: {deque:?}");

        // Traverse and build dependency graph
        while let Some(d) = deque.pop_front() {
            if visited.contains_key(&d) {
                continue;
            }

            let v = included_files.get(&d).unwrap(); // 👈 just borrow, don't remove yet
            let mut deps = HashSet::new();

            // Search for nested dependencies
            for (k, _) in &included_files {
                if k == &d {
                    continue; // avoid self-dep or reprocessing
                }

                let re = Regex::new(
                    &bars
                        .render("libcheck", &json!({"name": k}))
                        .map_to_string()?,
                )
                    .map_to_string_mess("Invalid regex for lib_check")?;

                if re.is_match(v) {
                    deps.insert(k.clone());
                    if !deque.contains(k) {
                        deque.push_back(k.clone()); // 👈 avoid pushing duplicates
                    }
                }
            }

            graph.insert(d.clone(), deps);
            visited.insert(d.clone(), extract_code_block(v));
        }

        println!("Graph: {graph:?}");


        let sorted_libs = topo_sort(&graph)?;

        println!("sorted_libs = {sorted_libs:?}");

        // Build the sorted lib_files map
        let lib_files = sorted_libs
            .into_iter()
            .rev()
            .filter_map(|k| visited.get(&k).map(|v| (k, v.clone())))
            .collect::<Vec<_>>(); // or regular BTreeMap/HashMap if order not needed beyond template

        let source_code = extract_code_block(&source_code);

        bars.register_template_string("modify", &self.code.modifier)
            .map_to_string()?;

        // Prepare context for the template
        let data = TemplateData {
            code: source_code,
            lib_files,
        };

        let res = bars.render("modify", &data).map_to_string()?;

        print!("{res}");

        Ok(res)
    }
}

fn topo_sort(graph: &HashMap<String, HashSet<String>>) -> Result<Vec<String>, String> {
    let mut in_degree = HashMap::new();
    let mut order = Vec::new();
    let mut queue = VecDeque::new();

    // Initialize in-degrees
    for (node, deps) in graph {
        in_degree.entry(node.clone()).or_insert(0);
        for dep in deps {
            *in_degree.entry(dep.clone()).or_insert(0) += 1;
        }
    }

    // Start with zero in-degree nodes
    for (node, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(node.clone());
        }
    }

    while let Some(node) = queue.pop_front() {
        order.push(node.clone());

        if let Some(deps) = graph.get(&node) {
            for dep in deps {
                if let Some(deg) = in_degree.get_mut(dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }
    }

    if order.len() == graph.len() {
        Ok(order)
    } else {
        Err("Cycle detected in dependency graph".into())
    }
}

#[tauri::command]
pub fn read_config(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let mut path = state.directory.clone();
    path.push("config.toml");

    let config: Config = if path.exists() {
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Error reading {:?}: {}", path, e))?;
        toml::from_str(&content).map_err(|e| format!("Error parsing config.toml: {}", e))?
    } else {
        // File doesn't exist: create with default content
        let default_config = Config::default();
        let toml_str = toml::to_string_pretty(&default_config)
            .map_err(|e| format!("Error serializing default config: {}", e))?;

        let mut file =
            fs::File::create(&path).map_err(|e| format!("Error creating config.toml: {}", e))?;
        file.write_all(toml_str.as_bytes())
            .map_err(|e| format!("Error writing config.toml: {}", e))?;

        default_config
    };

    state.config = config;

    Ok(())
}
