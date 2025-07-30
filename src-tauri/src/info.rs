use crate::judge::Verdict;
use crate::WINDOW;
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    name: String,
    group: String,
    url: String,
    interactive: bool,
    memory_limit: usize, // mb
    time_limit: usize,   // ms
    tests: Vec<Test>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Test {
    input: String,
    output: String,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Problem {
    pub title: String,
    pub url: String,
    pub memory_limit: usize,
    pub time_limit: usize,
}

impl Test {
    pub fn get_verdict(&self) -> Verdict {
        Verdict {
            answer: self.output.clone(),
            input: self.input.clone(),
            output: "".into(),
            stderr: "".into(),
            memory: 0.0,
            time: 0.0,
            status_id: 0,
            status: "NA".into(),
        }
    }
}

impl Info {
    pub fn get_problem(&self) -> Problem {
        Problem {
            title: self.name.clone(),
            memory_limit: self.memory_limit,
            time_limit: self.time_limit,
            url: self.url.clone(),
        }
    }

    pub fn get_verdicts(&self) -> Vec<Verdict> {
        self.tests.iter().map(|x| x.get_verdict()).collect()
    }
}

#[post("/")]
pub async fn get_info(req_body: web::Json<Info>) -> impl Responder {
    let window = WINDOW.get().expect("window-is-unavailable");
    window.emit("set-problem", req_body.get_problem()).unwrap();
    window
        .emit("set-verdicts", req_body.get_verdicts())
        .unwrap();
    HttpResponse::Ok()
}
