use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use actix_web_actors::ws;

use crate::config::Config;
use crate::tasks::TaskClient;
use crate::session::{SessionManager, TaskSession, TasksUpdated};


pub struct AppState {
    pub client: Mutex<TaskClient>,
    pub session_manager: Addr<SessionManager>,
}

pub struct Server {}

impl Server {
    pub async fn start(config: &Config, client: TaskClient) -> std::io::Result<()> {
        env_logger::init();

        let state = web::Data::new(AppState {
            // FIXME: get rid of unwrap - MCL - 2020-11-17
            client: Mutex::new(client),
            session_manager: SessionManager::new().start(),
        });

        let mut server = HttpServer::new(move || {
            let api = web::scope("/api/v1")
                .service(tasks_list)
                .service(refresh_tasks);

            let socket_service = web::resource("/ws/").to(ws_index);

            App::new()
                .wrap(Logger::default())
                .wrap(Logger::new("%a %{User-Agent}i"))
                .app_data(state.clone())
                .service(api)
                .service(socket_service)
        });

        let port = config.port.clone().unwrap();

        for addr in config.bind.clone().unwrap() {
            server = server.bind(&format!("{}:{}", addr, port))?;
        }

        server.run().await
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
        Ok(_) => {
            let addr = &data.session_manager;
            addr.do_send(TasksUpdated);
            HttpResponse::Ok()
        },
        // FIXME: actual error message - MCL - 2020-11-17
        Err(_) => HttpResponse::InternalServerError(),
    }
}

pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> std::result::Result<HttpResponse, actix_web::Error> {
    ws::start(
        TaskSession {
            hb: Instant::now(),
            addr: data.session_manager.clone(),
            id: "".to_string(),
        },
        &req,
        stream,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    // #[actix_rt::test]
    // async fn test_index_ok() {
    //     let req = test::TestRequest::with_header("content-type", "text/plain").to_http_request();
    //     let resp = index(req).await;
    //     assert_eq!(resp.status(), http::StatusCode::OK);
    // }
}
