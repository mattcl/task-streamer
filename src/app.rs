use std::sync::Mutex;
use std::time::Instant;

use actix::prelude::*;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{get, http, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::config::Config;
use crate::schema::Topic;
use crate::session::{SessionManager, TaskSession, TasksUpdated, TopicUpdated};

use task_hookrs::task::Task;

pub struct AppState {
    pub topic: Mutex<Topic>,
    pub tasks: Mutex<Vec<Task>>,
    pub session_manager: Addr<SessionManager>,
    pub api_key: String,
}

pub struct Server {}

impl Server {
    pub async fn start(config: Config) -> std::io::Result<()> {
        env_logger::init();

        let state = web::Data::new(AppState {
            topic: Mutex::new(Topic::default()),
            tasks: Mutex::new(Vec::new()),
            session_manager: SessionManager::new().start(),
            api_key: config.server.api_key.unwrap(),
        });

        let mut server = HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600);

            let api = web::scope("/api/v1")
                .service(get_tasks)
                .service(set_tasks)
                .service(get_topic)
                .service(set_topic);

            let socket_service = web::resource("/ws/").to(ws_index);

            App::new()
                .wrap(Logger::default())
                .wrap(Logger::new("%a %{User-Agent}i"))
                .wrap(cors)
                .app_data(state.clone())
                .service(api)
                .service(socket_service)
        });

        let port = config.server.port.unwrap();

        for addr in config.server.bind.unwrap() {
            server = server.bind(&format!("{}:{}", addr, port))?;
        }

        server.run().await
    }
}

#[get("/tasks")]
async fn get_tasks(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(&data.tasks)
}

#[post("/tasks")]
async fn set_tasks(
    data: web::Data<AppState>,
    item: web::Json<Vec<Task>>,
    auth: BearerAuth,
) -> impl Responder {
    if auth.token() == data.api_key.as_str() {
        let mut tasks = data.tasks.lock().unwrap();
        *tasks = item.0;
        let addr = &data.session_manager;
        addr.do_send(TasksUpdated);
        return HttpResponse::Ok();
    }
    HttpResponse::Unauthorized()
}

#[get("/topic")]
async fn get_topic(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(&data.topic)
}

#[post("/topic")]
async fn set_topic(
    data: web::Data<AppState>,
    item: web::Json<Topic>,
    auth: BearerAuth,
) -> impl Responder {
    if auth.token() == data.api_key.as_str() {
        let mut topic = data.topic.lock().unwrap();
        *topic = item.0;
        let addr = &data.session_manager;
        addr.do_send(TopicUpdated);
        return HttpResponse::Ok();
    }
    HttpResponse::Unauthorized()
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
