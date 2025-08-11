use crate::{config::Config, info::Problem, judge::Verdict, utils::ResultTrait, Language};
use chrono::Local;
use notify::{FsEventWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::{BufReader, Write},
    ops::Deref,
    path::PathBuf,
    process::Command,
    str::FromStr,
    sync::{Arc, Mutex, RwLock},
    time,
};
use tauri::{Manager, State};
use wait_timeout::ChildExt;

// Windows-specific imports
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use arboard::Clipboard;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000; // Prevents opening a new window

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AppState {
    pub directory: PathBuf,
    pub language_id: usize,
    #[serde(default, skip)]
    pub config: Config,
    #[serde(default, skip)]
    pub languages: HashMap<String, Language>,
    #[serde(default, skip)]
    pub problem: Problem,
    #[serde(default, skip)]
    pub verdicts: Vec<Verdict>,
    #[serde(default, skip)]
    pub watcher: Option<Arc<RwLock<FsEventWatcher>>>,
}

impl AppState {
    pub fn from_dir(dir: PathBuf) -> std::io::Result<Self> {
        create_dir_all(&dir)?;

        let mut file_path = dir.clone();
        file_path.push("cp_state.json");

        let res = if file_path.exists() {
            serde_json::from_reader(BufReader::new(File::open(file_path)?))?
        } else {
            let mut f = File::create(file_path)?;
            let state = AppState::default();
            f.write_fmt(format_args!("{}", serde_json::to_string(&state)?))?;
            state
        };

        Ok(res)
    }

    pub fn get_language(&self) -> Result<Language, String> {
        let language = self
            .languages
            .get(&self.language_id.to_string())
            .ok_or("language not found in langauges")?
            .clone();
        Ok(language)
    }
}

#[tauri::command]
pub fn get_directory(state: State<'_, Mutex<AppState>>) -> String {
    state.lock().unwrap().directory.to_str().unwrap().into()
}

#[tauri::command]
pub fn set_directory(directory: String, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let dir = PathBuf::from_str(&directory).map_to_string()?;
    state.lock().unwrap().directory = dir;
    Ok(())
}

#[tauri::command]
pub fn get_problem(state: State<'_, Mutex<AppState>>) -> Problem {
    state.lock().unwrap().problem.clone()
}

#[tauri::command]
pub async fn set_problem(
    problem: Problem,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    state.lock().unwrap().problem = problem;
    // if state.lock().unwrap().config.toggle.create_file {
    create_file(state).await
    // }
    // Ok(())
}

#[tauri::command]
pub fn get_verdicts(state: State<'_, Mutex<AppState>>) -> Vec<Verdict> {
    state.lock().unwrap().verdicts.clone()
}

#[tauri::command]
pub fn set_verdicts(verdicts: Vec<Verdict>, state: State<'_, Mutex<AppState>>) {
    state.lock().unwrap().verdicts = verdicts
}

#[tauri::command]
pub fn save_state(
    handle: tauri::AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut file_path = handle.path().app_config_dir().map_to_string()?;
    file_path.push("cp_state.json");

    let state = state.lock().unwrap().deref().clone();

    let mut f = File::create(file_path).map_to_string()?;
    f.write_fmt(format_args!(
        "{}",
        serde_json::to_string(&state).map_to_string()?
    ))
    .map_to_string()?;

    Ok(())
}

#[tauri::command]
pub async fn copy_code(app_state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = app_state.lock().unwrap();
    let problem = state.problem.clone();
    let code = state.config.get_final_code(&problem, &state.directory)?;
    let mut clipboard = Clipboard::new().map_to_string()?;
    clipboard.set_text(code).map_to_string()?;
    Ok(())
}

#[tauri::command]
pub async fn create_file(app_state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = app_state.lock().unwrap().clone();
    let config = &state.config;

    let file_path = state
        .config
        .get_file_path(&state.problem, &state.directory)?;
    create_dir_all(&file_path.parent().ok_or("invalid file path".to_string())?).map_to_string()?;

    if !file_path.exists() {
        let mut f = File::create_new(&file_path).map_to_string()?;
        let formatted_time = Local::now().format("%Y/%m/%d %H:%M"); // This is a Display object
        f.write_fmt(format_args!(
            "{} Created by {} at {}\n{} {}\n{}",
            state.get_language()?.comment, // e.g., "//"
            config.author,                 // e.g., "Ayush Biswas"
            formatted_time,                // formatted as "2024/12/25 14:07"
            state.get_language()?.comment, // e.g., "//"
            state.problem.url,             // problem URL
            config.get_template(&state.directory)
        ))
        .map_to_string()?;
    }

    if config.toggle.run_on_save {
        let watcher = state.watcher.unwrap();
        let mut watcher = watcher.write().unwrap();
        watcher
            .watch(&file_path, RecursiveMode::NonRecursive)
            .map_to_string()?;
    }

    let mut cmd = Command::new(config.editor.clone())
        .arg(&file_path)
        .spawn()
        .map_to_string()?;
    cmd.wait_timeout(time::Duration::from_secs(1))
        .map_to_string()?;

    Ok(())
}
