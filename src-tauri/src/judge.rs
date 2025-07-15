use crate::{state::AppState, utils::*, Language, WINDOW};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, create_dir_all, remove_dir_all},
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{Emitter, State};
use uuid::Uuid;
use wait_timeout::ChildExt;

// Windows-specific imports
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000; // Prevents opening a new window

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Verdict {
    pub input: String,
    pub output: String,
    pub answer: String,
    pub status_id: usize,
    pub status: String,
    pub time: f32,
    pub memory: f32,
}

#[tauri::command]
pub async fn test(
    app_state: State<'_, Mutex<AppState>>,
    handle: tauri::AppHandle,
) -> Result<(), String> {
    let state = app_state.lock().unwrap();

    // creating a temporary directory
    let mut dir = std::env::temp_dir();
    dir.push(Uuid::new_v4().to_string());

    let language = state.get_language()?;

    let mut file_path = dir.clone();
    file_path.push(&language.source_file);

    let source_file = state
        .config
        .get_final_code(&state.problem, &state.directory)?;

    // copy the final code into the temporary directory
    create_dir_all(&dir).map_to_string()?;
    fs::write(file_path, source_file).map_to_string()?;

    let mut verdicts = state.verdicts.clone();
    for v in &mut verdicts {
        v.status = "Compiling".into();
        v.status_id = 1;
    }
    handle.emit("set-verdicts", &verdicts).map_to_string()?;

    // First try to compiler and if compilation error occurs then return
    if let Err(e) = compile(&language, &dir) {
        for v in &mut verdicts {
            v.output = e.clone();
            v.status = "Compilation Error".into();
            v.status_id = 6;
        }
        handle.emit("set-verdicts", &verdicts).map_to_string()?;
    } else {
        for v in &mut verdicts {
            v.status = "Running".into();
            v.status_id = 2;
        }
        handle.emit("set-verdicts", &verdicts).map_to_string()?;

        let time_limit = state.problem.time_limit;
        let verdicts = run_all(&language, &dir, verdicts, time_limit)?;
        if verdicts.iter().all(|v| v.status == "Accepted") && state.config.toggle.submit_on_ac {
            WINDOW
                .get()
                .expect("could not find widow")
                .emit("submit", 0)
                .map_to_string()?;
        }
        handle.emit("set-verdicts", &verdicts).map_to_string()?;
    }

    remove_dir_all(dir).map_to_string()?;

    Ok(())
}

fn compile(language: &Language, dir: &Path) -> Result<bool, String> {
    if language.compiler_cmd.is_empty() {
        // If there is no compilation step then nothing to do
        return Ok(true);
    }

    #[cfg(windows)]
    let output = Command::new(&language.compiler_cmd)
        .current_dir(dir)
        .args(&language.compiler_args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_to_string()?;

    #[cfg(not(windows))]
    let output = Command::new(&language.compiler_cmd)
        .current_dir(dir)
        .args(&language.compiler_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_to_string()?;

    if output.status.success() {
        Ok(true)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string()
            + String::from_utf8_lossy(&output.stdout).to_string().as_str())
    }
}

fn run_all(
    language: &Language,
    dir: &Path,
    verdicts: Vec<Verdict>,
    time_limit: usize,
) -> Result<Vec<Verdict>, String> {
    let mut res = vec![];
    for v in verdicts {
        res.push(run(language, dir, v, time_limit)?);
    }
    Ok(res)
}

fn run(
    language: &Language,
    dir: &Path,
    mut verdict: Verdict,
    time_limit: usize,
) -> Result<Verdict, String> {
    #[cfg(debug_assertions)]
    println!("dir: {}", dir.to_str().unwrap());
    let run_cmd = &language.run_cmd;

    #[cfg(target_os = "windows")]
    {
        if !language.run_cmd_win.is_empty() {
            run_cmd = &language.run_cmd_win;
        }
    }

    #[cfg(debug_assertions)]
    println!("run_cmd: {}", run_cmd);

    // Create command with platform-specific options
    #[cfg(windows)]
    let mut child = Command::new(resolve_path(dir, run_cmd))
        .current_dir(dir)
        .args(&language.run_args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_to_string()?;

    #[cfg(not(windows))]
    let mut child = Command::new(resolve_path(dir, run_cmd))
        .current_dir(dir)
        .args(&language.run_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_to_string()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(verdict.input.as_bytes()).map_to_string()?;
    }

    let start = Instant::now();
    let status = child.wait_timeout(Duration::from_millis(time_limit as u64));
    verdict.time = start.elapsed().as_secs_f32() * 1000.0;

    match status {
        Ok(Some(exit_status)) => {
            let mut stdout = String::new();
            if let Some(mut s) = child.stdout.take() {
                s.read_to_string(&mut stdout).map_to_string()?;
            }
            let mut stderr = String::new();
            if let Some(mut s) = child.stderr.take() {
                s.read_to_string(&mut stderr).map_to_string()?;
            }

            if !exit_status.success() {
                verdict.output = stderr;
                verdict.status_id = 11;
                verdict.status = "Runtime Error (NZEC)".into();
            } else {
                verdict.output = stdout;
                if check(&verdict.answer, &verdict.output) {
                    verdict.status = "Accepted".into();
                    verdict.status_id = 3;
                } else {
                    verdict.status = "Wrong Answer".into();
                    verdict.status_id = 4;
                }
            }
        }
        Ok(None) => {
            child.kill().map_to_string()?;
            child.wait().map_to_string()?; // Wait to reap child
            verdict.status = "Time Limit Exceeded".into();
            verdict.status_id = 5;
        }
        Err(e) => {
            verdict.output = e.to_string();
            verdict.status_id = 7;
            verdict.status = "Runtime Error".into();
        }
    }

    Ok(verdict)
}

fn check(output: &String, answer: &String) -> bool {
    output
        .trim()
        .split('\n')
        .map(|x| x.trim())
        .zip(answer.trim().split('\n').map(|x| x.trim()))
        .all(|(x, y)| x == y)
}
