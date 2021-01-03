use log::{debug, error, info};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct TasksUpdated;

#[derive(Message)]
#[rtype(result = "()")]
pub struct TopicUpdated;

#[derive(Message)]
#[rtype(String)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}

pub struct TaskSession {
    pub hb: Instant,
    pub addr: Addr<SessionManager>,
    pub id: String,
}

pub struct SessionManager {
    sessions: HashMap<String, Recipient<Message>>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: HashMap::new(),
        }
    }

    pub fn notify_update(&self, msg: &str) {
        for session in self.sessions.values() {
            match session.do_send(Message(msg.to_string())) {
                // TODO: something better - MCL - 2020-11-24
                Ok(_) => (),
                Err(_) => (),
            };
        }
    }
}

impl Actor for SessionManager {
    type Context = Context<Self>;
}

impl Handler<Connect> for SessionManager {
    type Result = String;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        debug!("New session established");

        let id = uuid::Uuid::new_v4().to_string();

        self.sessions.insert(id.clone(), msg.addr);

        id
    }
}

impl Handler<Disconnect> for SessionManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        debug!("Disconnecting {}", msg.id);

        self.sessions.remove(&msg.id);

        ()
    }
}

impl Handler<TasksUpdated> for SessionManager {
    type Result = ();

    fn handle(&mut self, _: TasksUpdated, _: &mut Context<Self>) -> Self::Result {
        info!("Notifying tasks updated");

        self.notify_update("tasks-updated");

        ()
    }
}

impl Handler<TopicUpdated> for SessionManager {
    type Result = ();

    fn handle(&mut self, _: TopicUpdated, _: &mut Context<Self>) -> Self::Result {
        info!("Notifying topic updated");

        self.notify_update("topic_updated");

        ()
    }
}

impl TaskSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                info!("Websocket Client heartbeat failed, disconnecting!");

                act.addr.do_send(Disconnect { id: act.id.clone() });

                ctx.stop();

                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for TaskSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.addr
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // failed to connect to session manager
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(Disconnect {
            id: self.id.clone(),
        });
        Running::Stop
    }
}

impl Handler<Message> for TaskSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for TaskSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        debug!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(_) => {}
            ws::Message::Binary(_) => error!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
