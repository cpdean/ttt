#![allow(unused_variables)]
extern crate byteorder;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate rand;
extern crate serde;
extern crate tokio_core;
extern crate tokio_io;

extern crate actix;
extern crate actix_web;

use std::time::{Duration, Instant};

//use actix::*;
use actix::fut;
use actix::{
    Actor, ActorContext, ActorFuture, Addr, Arbiter, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web::server::HttpServer;
use actix_web::{fs, http, ws, App, Error, HttpRequest, HttpResponse};

mod server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// This is our websocket route state, this state is shared with all route
/// instances via `HttpContext::state()`
struct WsChatSessionState {
    addr: Addr<server::ChatServer>,
}

/// Entry point for our route
fn chat_route(req: &HttpRequest<WsChatSessionState>) -> Result<HttpResponse, Error> {
    ws::start(
        req,
        WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "Main".to_owned(),
        },
    )
}

struct WsChatSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// joined room
    room: String,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self, WsChatSessionState>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        ctx.state()
            .addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // notify chat server
        ctx.state().addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::GameMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::GameMessage, ctx: &mut Self::Context) {
        match msg {
            server::GameMessage::Chat(chat) => {
                let f = serde_json::to_string(&chat);
                match f {
                    Ok(json_string) => {
                        println!("doing a message {}", &json_string);
                        ctx.text(json_string);
                    }
                    Err(e) => {
                        println!("error of {} trying to deal with {:?}", e, &chat);
                    }
                }
            }
            server::GameMessage::Turn(turn) => {
                let f = serde_json::to_string(&turn);
                match f {
                    Ok(json_string) => {
                        println!("doing a turn {}", &json_string);
                        ctx.text(json_string);
                    }
                    Err(e) => {
                        println!("error of {} trying to deal with {:?}", e, &turn);
                    }
                }
            }
        }
    }
}

// old chat based handler /// WebSocket message handler
/*
impl StreamHandler<ws::Message, ws::ProtocolError> for WsChatSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // Send ListRooms message to chat server and wait for
                            // response
                            println!("List rooms");
                            ctx.state()
                                .addr
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ok(())
                                })
                                .wait(ctx)
                            // .wait(ctx) pauses all events in context,
                            // so actor wont receive any new messages until it get list
                            // of rooms back
                        }
                        "/join" => {
                            if v.len() == 2 {
                                self.room = v[1].to_owned();
                                ctx.state().addr.do_send(server::Join {
                                    id: self.id,
                                    name: self.room.clone(),
                                });

                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.name = Some(v[1].to_owned());
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                    }
                } else {
                    let msg = if let Some(ref name) = self.name {
                        format!("{}: {}", name, m)
                    } else {
                        m.to_owned()
                    };
                    // send message to chat server
                    ctx.state().addr.do_send(server::ClientMessage {
                        id: self.id,
                        msg: msg,
                        room: self.room.clone(),
                    })
                }
            }
            ws::Message::Binary(bin) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}
*/

impl StreamHandler<ws::Message, ws::ProtocolError> for WsChatSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let general_message: server::JsonGeneralMessage = match serde_json::from_str(&text)
                {
                    Ok(gen_msg) => gen_msg,
                    Err(e) => {
                        eprintln!("bad json parsing: {:?}", e);
                        ctx.text(format!("bad json parsing: {:?}", e));
                        return;
                    }
                };
                match general_message.event_type.as_ref() {
                    "chatmessage" => {
                        println!("a chat");
                        ctx.state().addr.do_send(server::ClientMessage {
                            id: self.id,
                            // TODO: remove .clone by moving this branch down into the ChatServer
                            // handler
                            event_type: general_message.event_type.clone(),
                            msg: general_message.data,
                            room: self.room.clone(),
                        });
                    }
                    "move" => {
                        println!("a game move");
                        ctx.state().addr.do_send(server::ClientMessage {
                            id: self.id,
                            event_type: general_message.event_type.clone(),
                            msg: general_message.data,
                            room: self.room.clone(),
                        });
                    }
                    e_type => {
                        println!("whatt???? {} ", e_type);
                        ctx.state().addr.do_send(server::ClientMessage {
                            id: self.id,
                            event_type: general_message.event_type.clone(),
                            msg: general_message.data,
                            room: self.room.clone(),
                        });
                    }
                }
            }
            ws::Message::Binary(bin) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}

impl WsChatSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self, WsChatSessionState>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                ctx.state().addr.do_send(server::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}

fn main() {
    let _ = env_logger::init();
    let sys = actix::System::new("websocket-example");

    // Start chat server actor in separate thread
    let chat_server = Arbiter::start(|_| server::ChatServer::default());

    // Create Http server with websocket support
    HttpServer::new(move || {
        // Websocket sessions state
        let state = WsChatSessionState {
            addr: chat_server.clone(),
        };

        App::with_state(state)
            // redirect to websocket.html
            .resource("/", |r| {
                r.method(http::Method::GET).f(|_| {
                    HttpResponse::Found()
                        .header("LOCATION", "/static/websocket.html")
                        .finish()
                })
            })
            // websocket
            .resource("/ws/", |r| r.route().f(chat_route))
            // static resources
            .handler("/static/", fs::StaticFiles::new("static/").unwrap())
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}

/*
cargo check
cargo test
*/

#[cfg(test)]
mod tests {
    // how to test it
    use actix::Arbiter;
    use actix_web::*;
    use futures::Stream;

    #[test]
    fn test_main() {
        let sys = actix::System::new("websocket-example");
        let chat_server = Arbiter::start(|_| super::server::ChatServer::default());
        // Start chat server actor in separate thread
        let mut srv = test::TestServer::build_with_state(move || {
            // Websocket sessions state
            super::WsChatSessionState {
                addr: chat_server.clone(),
            }
        })
        .start(|app| {
            app.handler(|req| {
                ws::start(
                    req,
                    super::WsChatSession {
                        id: 0,
                        hb: super::Instant::now(),
                        room: "Main".to_owned(),
                    },
                )
            })
        });

        let (reader, mut writer) = srv.ws().unwrap();
        writer.text("{\"foo\": 111}");

        /*
        let (item, reader) = srv.execute(reader.into_future()) {
            Ok((item, reader)) => {
                core::mem::drop(sys);
                core::mem::drop(srv);
                1
            }
            Err(e) => {
                core::mem::drop(sys);
                core::mem::drop(srv);
                1
            }
        };
        */
        let (item, reader) = srv.execute(reader.into_future()).unwrap();
        assert_eq!(item, Some(ws::Message::Text("text".to_owned())));
    }
}
