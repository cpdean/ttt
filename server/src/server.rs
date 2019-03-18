//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize, Message)]
pub struct JsonGeneralMessage {
    pub event_type: String,
    pub data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Message)]
pub struct TicTacToeGame {
    pub player1: Option<usize>,
    pub player2: Option<usize>,
    pub current_player_turn: Option<usize>,
    pub grid: Vec<Vec<usize>>,
    pub winner: Option<usize>,
}

impl TicTacToeGame {
    pub fn new() -> Self {
        TicTacToeGame {
            player1: None,
            player2: None,
            current_player_turn: None,
            grid: vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]],
            winner: None,
        }
    }

    pub fn add_player(&mut self, id: usize) {
        // make the joiner a player
        if self.player1.is_none() {
            self.player1 = Some(id);
        } else if self.player2.is_none() {
            self.player2 = Some(id);
        }
    }

    pub fn remove_player(&mut self, id: usize) {
        if self.player1 == Some(id) {
            self.player1 = None;
        } else if self.player2 == Some(id) {
            self.player2 = None;
        }
    }
}

#[derive(Debug, Message)]
pub enum GameMessage {
    Chat(ChatMessage),
    Turn(GameStateMessage),
}

/// Chat server sends this messages to session
#[derive(Debug, Serialize, Deserialize, Message)]
pub struct ChatMessage {
    pub event_type: String,
    pub content: String,
    pub message_count: usize,
}

/// broadcasting game state
#[derive(Debug, Serialize, Deserialize, Message)]
pub struct GameStateMessage {
    pub event_type: String,
    pub content: TicTacToeGame,
}

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<GameMessage>,
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Message)]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// game turn or chat message
    pub event_type: String,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
pub struct Join {
    /// Client id
    pub id: usize,
    /// Room name
    pub name: String,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<GameMessage>>,
    rooms: HashMap<String, ChatRoom>,
    rng: ThreadRng,
}

struct ChatRoom {
    sessions_subscribed_to_room: HashSet<usize>,
    message_count: usize,
    game_state: TicTacToeGame,
}

impl ChatRoom {
    pub fn new() -> Self {
        ChatRoom {
            sessions_subscribed_to_room: HashSet::new(),
            message_count: 0,
            game_state: TicTacToeGame::new(),
        }
    }
}

impl Default for ChatServer {
    fn default() -> ChatServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), ChatRoom::new());

        ChatServer {
            sessions: HashMap::new(),
            rooms: rooms,
            rng: rand::thread_rng(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Message)]
pub struct GameTurnMessage {
    position: Vec<usize>,
    player: String,
}

// TODO: i'm clearly calling it below
#[allow(dead_code)]
fn advance_turn(cm: GameTurnMessage, game_state: TicTacToeGame) -> TicTacToeGame {
    let mut gm = game_state;
    let x = cm.position[0];
    let y = cm.position[1];
    println!("{}", cm.player);
    let symbol = if cm.player == "player1" { 1 } else { 2 };
    gm.grid[y][x] = symbol;
    gm
}

impl ChatServer {
    fn send_turn(&mut self, room_name: &str, message: &str, skip_id: usize) {
        println!("sending the turn now");
        if let Some(room) = self.rooms.get_mut(room_name) {
            let gameturn: GameTurnMessage = serde_json::from_str(&message).unwrap();
            let next_turn = advance_turn(gameturn, room.game_state.clone());
            let next_turn_json = serde_json::to_string(&next_turn).unwrap();
            room.game_state = next_turn.clone();
            for id in &room.sessions_subscribed_to_room {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(&id) {
                        room.message_count += 1;
                        let _ = addr.do_send(GameMessage::Turn(GameStateMessage {
                            event_type: "board".to_owned(),
                            content: next_turn.clone(),
                        }));
                    }
                }
            }
        }
    }

    fn send_chat(&mut self, room_name: &str, message: &str, skip_id: usize) {
        if let Some(room) = self.rooms.get_mut(room_name) {
            for id in &room.sessions_subscribed_to_room {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(&id) {
                        room.message_count += 1;
                        let _ = addr.do_send(GameMessage::Chat(ChatMessage {
                            event_type: "chat".to_owned(),
                            content: message.to_owned(),
                            message_count: room.message_count,
                        }));
                    }
                }
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        self.send_chat(&"Main".to_owned(), "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to Main room
        self.rooms
            .get_mut(&"Main".to_owned())
            .unwrap()
            .sessions_subscribed_to_room
            .insert(id);

        let main_room = self.rooms.get_mut(&"Main".to_owned()).unwrap();

        // make the joiner a player
        main_room.game_state.add_player(id);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, room) in &mut self.rooms {
                if room.sessions_subscribed_to_room.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
                // also remove that player.
                room.game_state.remove_player(msg.id);
            }
        }
        // send message to other users
        for room in rooms {
            self.send_chat(&room, "Someone disconnected", 0);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        match msg.event_type.as_ref() {
            "chatmessage" => self.send_chat(&msg.room, &msg.msg.to_owned(), msg.id),
            "move" => self.send_turn(&msg.room, &msg.msg.to_owned(), msg.id),
            e_type => {
                println!("some kind of error???? {} ", e_type);
            }
        }
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, name } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, room) in &mut self.rooms {
            if room.sessions_subscribed_to_room.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            self.send_chat(&room, "Someone disconnected", 0);
        }

        if self.rooms.get_mut(&name).is_none() {
            self.rooms.insert(name.clone(), ChatRoom::new());
        }
        self.send_chat(&name, "Someone connected", id);
        self.rooms
            .get_mut(&name)
            .unwrap()
            .sessions_subscribed_to_room
            .insert(id);
    }
}
