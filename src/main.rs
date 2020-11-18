use std::collections::HashSet;
use std::sync::Mutex;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use crate::error::{Result, TSError};
use crate::tasks::TaskClient;

mod cli;
mod error;
mod tasks;

struct AppState {
    client: Mutex<TaskClient>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = cli::cli();
    match matches.subcommand() {
        ("server", Some(server_matches)) => {
            let state = web::Data::new(AppState {
                // FIXME: get rid of unwrap - MCL - 2020-11-17
                client: Mutex::new(TaskClient::new().unwrap()),
            });

            let mut server = HttpServer::new(move || {
                let api = web::scope("/api/v1")
                    .service(tasks_list)
                    .service(refresh_tasks);
                App::new().app_data(state.clone()).service(api)
            });

            let port = server_matches.value_of("port").unwrap();

            if server_matches.is_present("bind") {
                let bind_addrs: HashSet<&str> = server_matches.values_of("bind").unwrap().collect();
                for addr in &bind_addrs {
                    server = server.bind(&format!("{}:{}", addr, port))?;
                }
            } else {
                server = server.bind(&format!("127.0.0.1:{}", port))?;
            }

            server.run().await
        }
        _ => Ok(()),
    }
}

#[get("/tasks")]
async fn tasks_list(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(&data.client.lock().unwrap().tasks)
}

#[post("/tasks")]
async fn refresh_tasks(data: web::Data<AppState>) -> impl Responder {
    let mut client = data.client.lock().unwrap();
    match client.refresh_tasks() {
        Ok(_) => HttpResponse::Ok(),
        // FIXME: actual error message - MCL - 2020-11-17
        Err(_) => HttpResponse::InternalServerError(),
    }
}
