use std::{fs::read_to_string, path::PathBuf, str::FromStr, sync::Mutex};

use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tauri_plugin_http::reqwest;

use crate::{file_name, utils::ResultTrait, AppState};

pub struct WebState {
    pub sol: Mutex<Option<Solution>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
    empty: bool,
    problem_name: String,
    url: String,
    source_code: String,
    language_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptySolution {
    empty: bool,
}

#[get("/getSubmit")]
pub async fn get_submit(data: web::Data<WebState>) -> impl Responder {
    let sol = data.sol.lock().unwrap().take();

    if sol.is_some() {
        let solution = sol.unwrap();

        #[cfg(debug_assertions)]
        println!("submitting solution");

        return HttpResponse::Ok().json(solution);
    }

    #[cfg(debug_assertions)]
    println!("no solution returning empty");

    HttpResponse::Ok().json(EmptySolution { empty: true })
}

#[post("/submit")]
pub async fn post_submit(sol: web::Json<Solution>, data: web::Data<WebState>) -> impl Responder {
    let _ = data.sol.lock().unwrap().insert(sol.0);

    #[cfg(debug_assertions)]
    println!("inserted solution into data");

    HttpResponse::Ok()
}

#[tauri::command]
pub async fn submit_solution(app_state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = app_state.lock().unwrap().clone();
    let mut file_path = PathBuf::from_str(&state.directory).map_to_string()?;
    file_path.push(state.get_language_dir());
    file_path.push(file_name(&state.problem.title));
    file_path.set_extension(state.get_language()?.get_extension());

    let source_code = read_to_string(file_path).map_to_string()?;

    let client = reqwest::Client::builder().build().map_to_string()?;

    let problem_name = state
        .problem
        .url
        .split('/')
        .rev()
        .take(2)
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .collect::<Vec<&str>>()
        .join("");

    let solution = Solution {
        empty: false,
        language_id: state.get_language().map_to_string()?.cf_id,
        problem_name,
        source_code,
        url: state.problem.url,
    };

    let post_request = client
        .post("http://localhost:27121/submit")
        .json(&solution)
        .build()
        .map_to_string()?;

    client.execute(post_request).await.map_to_string()?;

    Ok(())
}
