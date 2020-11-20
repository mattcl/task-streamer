use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;

use crate::tasks::TaskClient;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(20);

pub async fn ws_index(r: HttpRequest, stream: web::Payload) -> std::result::Result<HttpResponse, actix_web::Error> {
    println!("{:?}", r);
    let res = ws::start(TaskSocket::new(), &r, stream);
    println!("{:?}", res);
    res
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
pub struct TaskSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for TaskSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<std::result::Result<ws::Message, ws::ProtocolError>> for TaskSocket {
    fn handle(
        &mut self,
        msg: std::result::Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        // process websocket messages
        println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl TaskSocket {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

struct AppState {
    client: Mutex<TaskClient>,
}

pub struct Server {}

impl Server {
    pub async fn start(port: &str, interfaces: &HashSet<&str>) -> std::io::Result<()> {
        let state = web::Data::new(AppState {
            // FIXME: get rid of unwrap - MCL - 2020-11-17
            client: Mutex::new(TaskClient::new().unwrap()),
        });

        let mut server = HttpServer::new(move || {
            let api = web::scope("/api/v1")
                .service(tasks_list)
                .service(refresh_tasks);

            // let socket_service = web::resource("/ws/").route(web::get().to(ws_index));

            App::new()
                .app_data(state.clone())
                .service(api)
                // .service(socket_service)
        });

        for addr in interfaces {
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
        Ok(_) => HttpResponse::Ok(),
        // FIXME: actual error message - MCL - 2020-11-17
        Err(_) => HttpResponse::InternalServerError(),
    }
}
