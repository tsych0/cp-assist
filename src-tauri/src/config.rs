use handlebars::Handlebars;
use handlebars_misc_helpers::register;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{collections::HashMap, path::Path};
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
        match fs::read_to_string(template_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading template file: {}", e);
                String::new()
            }
        }
    }

    pub fn get_final_code(&self, problem: &Problem, dir: &Path) -> Result<String, String> {
        // Read source code
        let source_code = fs::read_to_string(self.get_file_path(problem, dir)?).map_to_string()?;
        let source_code = extract_code_block(&source_code);

        // Get included files content
        let mut included_files = self.get_included_files(dir)?;
        // let re = regex

        let mut deque = VecDeque::new();
        let mut visited = HashMap::new();

        let mut bars = Handlebars::new();
        register(&mut bars);
        bars.register_template_string("libcheck", &self.code.lib_check_regex)
            .map_to_string()?;

        // First see what files are required in the source code
        for (k, _) in &included_files {
            let re = Regex::new(
                &bars
                    .render("libcheck", &json!({"name": k}))
                    .map_to_string()?,
            )
            .map_to_string_mess("Invalid regex for lib_check")?;
            println!("regex: {}", re.as_str());
            if re.is_match(&source_code) {
                deque.push_back(k.clone());
            }
        }

        // Then dfs to see what are the dependencies
        while let Some(d) = deque.pop_front() {
            if visited.contains_key(&d) {
                continue;
            }
            let v = included_files.remove(&d).unwrap();
            visited.insert(d.clone(), v.clone());
            for (k, _) in &included_files {
                let re = Regex::new(
                    &bars
                        .render("libcheck", &json!({"name": k}))
                        .map_to_string()?,
                )
                .map_to_string_mess("Invalid regex for lib_check")?;
                if re.is_match(&v) {
                    deque.push_back(k.clone());
                }
            }
        }

        bars.register_template_string("modify", &self.code.modifier)
            .map_to_string()?;

        // Prepare context for the template
        let data = json!({
            "code": source_code,
            "lib_files": visited
        });

        Ok(bars.render("modify", &data).map_to_string()?)
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
