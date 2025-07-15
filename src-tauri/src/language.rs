use crate::{state::AppState, utils::ResultTrait};
use serde::{Deserialize, Serialize};
use std::fs;
use std::{
    collections::HashMap,
    process::{Command, Stdio},
    sync::Mutex,
    time::Duration,
};
use tauri::{path::BaseDirectory, Manager, State};
use wait_timeout::ChildExt;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000; // Prevents opening a new window

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Language {
    pub id: usize,
    pub cf_id: usize,
    pub name: String,
    #[serde(skip_serializing)]
    pub source_file: String,
    #[serde(skip_serializing)]
    pub compiler_cmd: String,
    #[serde(skip_serializing)]
    pub compiler_args: Vec<String>,
    #[serde(skip_serializing)]
    pub run_cmd: String,
    #[serde(skip_serializing)]
    pub run_cmd_win: String,
    #[serde(skip_serializing)]
    pub run_args: Vec<String>,
    #[serde(skip_serializing)]
    pub check_args: Vec<String>,
    #[serde(skip_serializing)]
    pub comment: String,
}

impl Language {
    pub fn check(&self) -> bool {
        let cmd = if self.compiler_cmd.is_empty() {
            &self.run_cmd
        } else {
            &self.compiler_cmd
        };

        #[cfg(windows)]
        let result = Command::new(cmd)
            .args(&self.check_args)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        #[cfg(unix)]
        let result = Command::new(cmd)
            .args(&self.check_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        if let Ok(mut o) = result {
            let _ = o.wait_timeout(Duration::from_secs(2));
            true
        } else {
            false
        }
    }
}

#[tauri::command]
pub async fn get_languages(
    state: State<'_, Mutex<AppState>>,
    handle: tauri::AppHandle,
) -> Result<Vec<Language>, String> {
    if state.lock().unwrap().languages.is_empty() {
        let path = handle
            .path()
            .resolve("Languages.toml", BaseDirectory::Config)
            .map_to_string_mess("Failed to resolve config path")?;

        // If config file doesn't exist, copy from resources
        if !path.exists() {
            let resource_path = handle
                .path()
                .resolve("Languages.toml", BaseDirectory::Resource)
                .map_to_string_mess("Languages.toml not found in resources")?;

            // Create config directory if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_to_string_mess("Failed to create config directory")?;
            }

            fs::copy(&resource_path, &path)
                .map_to_string_mess("Failed to copy to config directory")?;
        }

        // Read from config directory
        let content =
            fs::read_to_string(&path).map_to_string_mess(&format!("Error reading {:?}", path))?;
        let languages: HashMap<String, Language> =
            toml::from_str(&content).map_to_string_mess("Error parsing Languages.toml")?;

        state.lock().unwrap().languages = languages.into_iter().filter(|(_, v)| v.check()).collect()
    }

    let languages_map = state.lock().unwrap().languages.clone();

    let mut languages = vec![];
    for (id, mut language) in languages_map {
        language.id = id.parse().unwrap_or(0);
        languages.push(language);
    }

    Ok(languages)
}

#[tauri::command]
pub fn get_language(state: State<'_, Mutex<AppState>>) -> usize {
    state.lock().unwrap().language_id
}

#[tauri::command]
pub fn set_language(language_id: usize, state: State<'_, Mutex<AppState>>) {
    state.lock().unwrap().language_id = language_id
}
