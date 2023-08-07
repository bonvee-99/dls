mod commands;
mod util;

use crate::commands::Command;
use futures_channel::mpsc;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{future, SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Mutex};
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message as WS_Message, MaybeTlsStream, WebSocketStream,
};

use openssl::rsa::{Rsa, Padding};
use openssl::pkey::{PKey, Private, Public};
use openssl::encrypt::{Encrypter, Decrypter};

use base64;

// use ring::rand::SystemRandom;
// use ring::signature::{Ed25519KeyPair, KeyPair};

static GLOBAL_DATA: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});

fn generate_keypair() -> (PKey<Private>, Vec<u8>) {
    let rsa = Rsa::generate(2048).unwrap();
    let private_key = PKey::from_rsa(rsa).unwrap();
    let public_key: Vec<u8> = private_key.public_key_to_pem().unwrap();
    (private_key, public_key)
}

fn encrypt_data(public_key: &str, text: &str) -> String {
    let pkey = PKey::public_key_from_pem(public_key.as_bytes()).unwrap();

    let data = text.as_bytes();
    let mut encrypter = Encrypter::new(&pkey).unwrap();
    encrypter.set_rsa_padding(Padding::PKCS1).unwrap();
    // Create an output buffer
    let buffer_len = encrypter.encrypt_len(data).unwrap();
    let mut encrypted = vec![0; buffer_len];
    // Encrypt and truncate the buffer
    let encrypted_len = encrypter.encrypt(data, &mut encrypted).unwrap();
    encrypted.truncate(encrypted_len);
    base64::encode(&encrypted)
}

fn decrypt_data(private_key: &PKey<Private>, text: &str) -> String {
    let data = base64::decode(text).unwrap();
    // // Decrypt the data
    let mut decrypter = Decrypter::new(private_key).unwrap();
    decrypter.set_rsa_padding(Padding::PKCS1).unwrap();
    // // Create an output buffer
    let buffer_len = decrypter.decrypt_len(&data).unwrap();
    let mut decrypted = vec![0; buffer_len];
    // // Encrypt and truncate the buffer
    let decrypted_len = decrypter.decrypt(&data, &mut decrypted).unwrap();
    decrypted.truncate(decrypted_len);
    std::str::from_utf8(&decrypted).unwrap().to_string()
}

#[tokio::main]
async fn main() {
    let (private_key, public_key) = generate_keypair();
    let mut global_data = GLOBAL_DATA.lock().unwrap();
    // save your own public key
    global_data.insert("public_key".to_string(), std::str::from_utf8(public_key.as_slice()).unwrap().to_string());
    std::mem::drop(global_data);
    // TODO: encrypt secret message before sending
    // TODO: have server send SecretMessage type for secrets!
    
    // 1) user receives base64 encoded public key
    // 2) user changes it to bytes
    // 3) user creates pkey with it and saves it to map (user_id: pkey)
    // 4) user can then use pkey to encrypt data

    // TODO: try decrypting data that was encrypted with wrong public key!

    
    let (stdin_tx, stdin_rx) = mpsc::unbounded::<WS_Message>();
    let (write, read) = server_connection().await;

    let stdin_to_ws = tokio::spawn(write_text_to_server(write, stdin_rx));
    let listen_stream = tokio::spawn(listen_to_server(read, private_key));
    tokio::spawn(cli_prompt(stdin_tx));
    future::select(stdin_to_ws, listen_stream).await;
}

async fn server_connection() -> (
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WS_Message>,
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) {
    let connection_url = "ws://localhost:3000";
    let url = url::Url::parse(&connection_url).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    util::log_info("Connected to server");
    return ws_stream.split();
}

async fn write_text_to_server(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WS_Message>,
    mut stdin_rx: UnboundedReceiver<WS_Message>,
) {
    // TODO: once cli global controls such as ^C are implemented, it will be a buffer
    loop {
        let msg = stdin_rx.next().await.unwrap();
        write.send(msg).await.unwrap();
    }
}

async fn listen_to_server(read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, private_key: PKey<Private>) {
    read.for_each(|message| async {
        let data = message.unwrap().into_data();
        let parsed_data = String::from_utf8(data).unwrap();
        handle_server_data(parsed_data, &private_key);
        tokio::io::stdout().flush().await.unwrap();
    })
    .await;
}

pub fn help_command() {
    let commands = vec![
        (
            Command::Start,
            "Start a session to send and receive secrets",
        ),
        (Command::Create, "Create a room to send secrets to"),
        (Command::Join, "Join a room to receive secrets from"),
        (Command::Send, "Send a secret to a peer"),
        (Command::List, "List all the secrets you have received"),
        (Command::Help, "Print this help message"),
        (Command::Quit, "Quit the application"),
    ];
    println!(
        "\x1b[1m\x1b[37m------------------------LIST OF COMMANDS------------------------\x1b[0m"
    );
    for (command, description) in commands {
        let command_str = format!("{:<15}", command.to_string());
        println!("\x1b[1m\x1b[92m{} \x1b[0m{}", command_str, description);
    }
    println!(
        "\x1b[1m\x1b[37m----------------------------------------------------------------\x1b[0m"
    );
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
        return;
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
        }
        Command::Join => join_room(arguments, stdin_tx),
        Command::Send => {
            send_message(arguments, stdin_tx);
        }
        Command::List => todo!(),
        Command::Help => {
            help_command();
        }
        Command::Quit => {
            println!("Goodbye!");
            std::process::exit(0);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct UserPublicKey {
    user_id: String,
    public_key: String
}

#[derive(Serialize, Deserialize)]
enum ServerResponse {
    JoinRoom { room_id: String, text: String, public_keys: Vec<UserPublicKey> },
    CreateRoom { room_id: String, text: String },
    Message { text: String },
    SecretMessage { user_id: String, text: String },
    PublicKey { public_key: String, user_id: String, text: String }
}

fn handle_server_data(data: String, private_key: &PKey<Private>) {
    let server_data: ServerResponse = serde_json::from_str(&data).unwrap();

    let message = match server_data {
        ServerResponse::JoinRoom { room_id, text, public_keys } => {
            let mut global_data = GLOBAL_DATA.lock().unwrap();
            global_data.insert("room".to_string(), room_id);
            for user_key in public_keys {
                // right now there is only one other person in room
                let UserPublicKey { user_id: _, public_key } = user_key;
                global_data.insert("roommate".to_string(), public_key);
            }
            text
        },
        ServerResponse::CreateRoom { room_id, text } => {
            let mut global_data = GLOBAL_DATA.lock().unwrap();
            global_data.insert("room".to_string(), room_id);
            text
        },
        ServerResponse::Message { text } => text,
        ServerResponse::SecretMessage { user_id, text } => {
            let decrypted_message = decrypt_data(private_key, &text);
            format!("{}: {}", user_id, decrypted_message)
        },
        ServerResponse::PublicKey { public_key, user_id, text } => {
            let mut global_data = GLOBAL_DATA.lock().unwrap();
            global_data.insert("roommate".to_string(), public_key);
            text
        },
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
    public_key: Option<String>,
}

fn create_room(stdin_tx: &UnboundedSender<WS_Message>) {
    let global_data = GLOBAL_DATA.lock().unwrap();
    let public_key = global_data.get("public_key");
    match public_key {
        Some(key) => {
            let msg = ToServerMessage {
                message_type: Some("create".to_string()),
                secret_message: None,
                room_id: None,
                public_key: Some(key.to_string()),
            };
            let json_msg = serde_json::to_string(&msg).unwrap();
            stdin_tx.unbounded_send(WS_Message::text(json_msg)).unwrap();
        }
        None => {
            // public key doesnt exist
        }
    }
}

fn join_room(arguments: Option<&[&str]>, stdin_tx: &UnboundedSender<WS_Message>) {
    match arguments {
        Some(args) => {
            let global_data = GLOBAL_DATA.lock().unwrap();
            let public_key = global_data.get("public_key");
            match public_key {
                Some(key) => {
                    let msg = ToServerMessage {
                        message_type: Some("join".to_string()),
                        secret_message: None,
                        room_id: Some(args[0].to_string()),
                        public_key: Some(key.to_string()),
                    };
                    let json_msg = serde_json::to_string(&msg).unwrap();
                    stdin_tx.unbounded_send(WS_Message::text(json_msg)).unwrap();
                }
                None => {
                    // public key doesnt exist
                }
            }
        }
        None => {
            println!("missing arguments for join");
        }
    }
}

fn send_message(arguments: Option<&[&str]>, stdin_tx: &UnboundedSender<WS_Message>) {
    let global_data = GLOBAL_DATA.lock().unwrap();
    if let Some(room) = global_data.get("room") {
        match arguments {
            Some(args) => {
                if let Some(key) = global_data.get("roommate") {
                    let msg = ToServerMessage {
                        message_type: Some("secret".to_string()),
                        secret_message: Some(encrypt_data(key, args[0])),
                        room_id: Some(room.to_string()),
                        public_key: None,
                    };
                    let json_msg = serde_json::to_string(&msg).unwrap();
                    stdin_tx.unbounded_send(WS_Message::text(json_msg)).unwrap();
                } else {
                    println!("no public key for roommate found");
                }
            }
            None => {
                println!("missing arguments for send");
            }
        }
    } else {
        println!("must join room before sending messages");
    }
}
