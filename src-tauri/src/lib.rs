mod config;
mod info;
mod judge;
mod language;
mod state;
mod submit;
mod utils;

use actix_web::{web, App, HttpServer};
use config::read_config;
use info::*;
use judge::*;
use language::*;
use notify::Event;
use state::*;
use std::{
    sync::{mpsc, Arc, Mutex, OnceLock, RwLock},
    thread,
    time::{self, Duration},
};
use submit::*;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use crate::utils::ResultTrait;

pub static WINDOW: OnceLock<WebviewWindow> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            _ = WINDOW.set(window);

            let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
            let watcher = notify::recommended_watcher(tx).map_to_string()?;

            let handle = thread::spawn(move || {
                let mut previous = time::Instant::now();

                let window = WINDOW.get().expect("window-is-unavailable");
                for res in rx {
                    let now = time::Instant::now();
                    if now.duration_since(previous) < Duration::from_secs(1) {
                        continue;
                    }
                    println!("got something");
                    match res {
                        Ok(_event) => window.emit("test", 0).unwrap(),
                        Err(e) => println!("watch error: {:?}", e),
                    }
                    previous = now;
                }
            });

            drop(handle);

            let web_state = web::Data::new(WebState {
                sol: Mutex::new(None),
            });

            tauri::async_runtime::spawn(
                HttpServer::new(move || {
                    App::new()
                        .app_data(web_state.clone())
                        .service(get_info)
                        .service(get_submit)
                        .service(post_submit)
                })
                .bind(("127.0.0.1", 27121))?
                .run(),
            );

            let dir = app.path().app_config_dir().unwrap();
            let mut state = AppState::from_dir(dir).unwrap();
            state.watcher = Some(Arc::new(RwLock::new(watcher)));
            app.manage(Mutex::new(state));

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_window(app);
        }))
        .invoke_handler(tauri::generate_handler![
            get_directory,
            set_directory,
            set_language,
            get_language,
            get_languages,
            set_problem,
            get_problem,
            set_verdicts,
            get_verdicts,
            create_file,
            copy_code,
            read_config,
            save_state,
            submit_solution,
            test,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_window(app: &AppHandle) {
    let windows = app.webview_windows();

    if let Some(window) = windows.values().next() {
        window.set_skip_taskbar(false).unwrap(); // ✅ Ensure it appears in taskbar
        window.show().unwrap(); // ✅ Make sure the window is visible
        window.set_focus().unwrap(); // ✅ Bring it to front
    } else {
        panic!("Sorry, no window found");
    }
}
