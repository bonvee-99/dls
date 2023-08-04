mod commands;
mod util;

use futures_util::{future, SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message as WS_Message, WebSocketStream};
use futures_channel::mpsc;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::stream::{SplitSink, SplitStream};
use tokio::io;
use tokio::net::TcpStream;
use crate::commands::Command;
use serde::{Deserialize, Serialize};
use std::{sync::Mutex, collections::HashMap};
use once_cell::sync::Lazy;

static GLOBAL_DATA: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});

#[tokio::main]
async fn main() {
    let (stdin_tx, stdin_rx) = mpsc::unbounded::<WS_Message>();
    let (write, read) = server_connection().await;

    let stdin_to_ws = tokio::spawn(write_text_to_server(write, stdin_rx));
    let listen_stream = tokio::spawn(listen_to_server(read));
    tokio::spawn(cli_prompt(stdin_tx));
    future::select(stdin_to_ws, listen_stream).await;
}

async fn server_connection() -> (SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WS_Message>, SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    let connection_url = "ws://localhost:3000";
    let url = url::Url::parse(&connection_url).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    util::log_info("Connected to server");
    return ws_stream.split();
}

async fn write_text_to_server(mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WS_Message>, mut stdin_rx: UnboundedReceiver<WS_Message>) {
    // TODO: once cli global controls such as ^C are implemented, it will be a buffer
    loop {
        let msg = stdin_rx.next().await.unwrap();
        write.send(msg).await.unwrap();
    }
}

async fn listen_to_server(read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    read.for_each(|message| async {
        let data = message.unwrap().into_data();
        let parsed_data = String::from_utf8(data).unwrap();
        handle_server_data(parsed_data);
        tokio::io::stdout().flush().await.unwrap();
    }).await;
}

pub fn help_command() {
    let commands = vec![
        (Command::Start, "Start a session to send and receive secrets"),
        (Command::Create, "Create a room to send secrets to"),
        (Command::Join, "Join a room to receive secrets from"),
        (Command::Send, "Send a secret to a peer"),
        (Command::List, "List all the secrets you have received"),
        (Command::Help, "Print this help message"),
        (Command::Quit, "Quit the application"),
    ];
    println!("\x1b[1m\x1b[37m------------------------LIST OF COMMANDS------------------------\x1b[0m");
    for (command, description) in commands {
        let command_str = format!("{:<15}", command.to_string());
        println!("\x1b[1m\x1b[92m{} \x1b[0m{}", command_str, description);
    }
    println!("\x1b[1m\x1b[37m----------------------------------------------------------------\x1b[0m");
}

pub async fn cli_prompt(stdin_tx: UnboundedSender<WS_Message>) {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin);

    loop {
        let mut input = String::new();
        let _bytes_read = reader.read_line(&mut input).await.unwrap();

        handle_input(&input, &stdin_tx);
    }
}

fn handle_input(input: &str, stdin_tx: &UnboundedSender<WS_Message>) {
    let trimmed_input = input.trim();
    let parts: Vec<&str> = trimmed_input.split_whitespace().collect();

    // TODO: fix so we can handle secrets with spaces. Right now we are only taking the first
    // word after "send"
    if parts.is_empty() {
        println!("No input received. Please try again");
        return
    }
    let command = match Command::from_string(parts[0]) {
        Some(cmd) => cmd,
        None => {
            println!("\x1b[1m\x1b[95m\nUnknown command: {}\x1b[0m", input.trim());
            Command::Help
        }
    };

    let arguments = if parts.len() > 1 {
        Some(&parts[1..])
    } else {
        None
    };
    
    match command {
        Command::Start => todo!(),
        Command::Create => {
            create_room(stdin_tx);
        },
        Command::Join => join_room(arguments, stdin_tx),
        Command::Send => {
            send_message(arguments, stdin_tx);
        },
        Command::List => todo!(),
        Command::Help => {
            help_command();
        },
        Command::Quit => {
            println!("Goodbye!");
            std::process::exit(0);
        }
    }
}

#[derive(Serialize, Deserialize)]
enum ServerResponse {
    JoinRoom {
        room_id: String,
        text: String
    },
    Message {
        text: String
    }
}

// #[derive(Serialize, Deserialize)]
// struct CreateRoomResult {
//     room_id: String,
//     text: String
// }

fn handle_server_data(data: String) {
    // println!("data: {}", data);
    // TODO: fix so server is sending right data in this form
    let server_data: ServerResponse = serde_json::from_str(&data).unwrap();

    let message = match server_data {
        ServerResponse::JoinRoom { room_id, text } => {
            let mut global_data = GLOBAL_DATA.lock().unwrap();
            global_data.insert("room".to_string(), room_id);
            text
        },
        ServerResponse::Message { text } => text,
    };
    println!("\x1b[1m\x1b[95m>>\x1b[0m");
    println!("\x1b[1m\x1b[36m{:?}\x1b[0m", message);
    println!("\x1b[1m\x1b[95m<<\x1b[0m");
}

#[derive(Serialize, Deserialize)]
struct ToServerMessage {
    // TODO: change this
    message_type: Option<String>,
    room_id: Option<String>,
    secret_message: Option<String>,
    key: Option<String>
}

fn create_room(stdin_tx: &UnboundedSender<WS_Message>) {
    let msg = ToServerMessage {
        message_type: Some("create".to_string()),
        secret_message: None,
        room_id: None,
        key: None
    };
    let json_msg = serde_json::to_string(&msg).unwrap();
    stdin_tx.unbounded_send(WS_Message::text(json_msg)).unwrap();
}

fn join_room(arguments: Option<&[&str]>, stdin_tx: &UnboundedSender<WS_Message>) {
    match arguments {
        Some(args) => {
            let msg = ToServerMessage {
                message_type: Some("join".to_string()),
                secret_message: None,
                // TODO: send the actual room id!
                room_id: Some(args[0].to_string()),
                key: None
            };
            let json_msg = serde_json::to_string(&msg).unwrap();
            stdin_tx.unbounded_send(WS_Message::text(json_msg)).unwrap();
        },
        None => {
            println!("missing arguments for join");
        },
    }
}

// TODO: cache the room so we don't need to use it as an argument (will need to use a lock ?)
// right now do $ send <message> <room_id>
// TODO: fix so we don't crash when user doesnt pass in the room id (or just fix so we dont need
// to pass the room id
fn send_message(arguments: Option<&[&str]>, stdin_tx: &UnboundedSender<WS_Message>) {
    let mut global_data = GLOBAL_DATA.lock().unwrap();
    if let Some(room) = global_data.get("room") {
        match arguments {
            Some(args) => {
                // encrypt message
                // stringify message
                // { type: send, message: <encrypted-message> }
                // { type: create }
                // { type: join, room: <room> }
                let msg = ToServerMessage {
                    message_type: Some("secret".to_string()),
                    secret_message: Some(args[0].to_string()),
                    // TODO: send the actual room id!
                    room_id: Some(room.to_string()),
                    key: None
                };
                let json_msg = serde_json::to_string(&msg).unwrap();
                stdin_tx.unbounded_send(WS_Message::text(json_msg)).unwrap();
            },
            None => {
                println!("missing arguments for send");
            },
        }
    } else {
        println!("must join room before sending messages");
    }
}

// end to end encryption possibilities:
// (1) Diffie–Hellman key exchange (symmetric key ?)

// (2) each user has public key to encrypt their messages
