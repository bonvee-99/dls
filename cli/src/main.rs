mod commands;
mod util;

use futures_util::{future, SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
use futures_channel::mpsc;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::stream::{SplitSink, SplitStream};
use tokio::io;
use tokio::net::TcpStream;
use crate::commands::Command;

#[tokio::main]
async fn main() {
    let (stdin_tx, stdin_rx) = mpsc::unbounded::<Message>();
    let (write, read) = server_connection().await;

    let stdin_to_ws = tokio::spawn(write_text_to_server(write, stdin_rx));
    let listen_stream = tokio::spawn(listen_to_server(read));
    tokio::spawn(cli_prompt(stdin_tx));
    future::select(stdin_to_ws, listen_stream).await;
}

// establish connection to server
async fn server_connection() -> (SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    let connection_url = "ws://localhost:3000";
    let url = url::Url::parse(&connection_url).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    util::log_info("Connected to server");
    return ws_stream.split();
}

async fn write_text_to_server(mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, mut stdin_rx: UnboundedReceiver<Message>) {
    // TODO: once cli global controls such as ^C are implemented, it will be a buffer
    loop {
        let msg = stdin_rx.next().await.unwrap();
        write.send(msg).await.unwrap();
    }
}

async fn listen_to_server(read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    read.for_each(|message| async {
        let data = message.unwrap().into_data();
        println!("\x1b[1m\x1b[95m>>\x1b[0m");
        println!("\x1b[1m\x1b[36m{:?}\x1b[0m", String::from_utf8(data).unwrap());
        println!("\x1b[1m\x1b[95m<<\x1b[0m");
        tokio::io::stdout().flush().await.unwrap();
    }).await;
}

pub fn help_command() {
    let commands = vec![
        (Command::Start, "Start a session to send and receive secrets"),
        (Command::CreateRoom, "Create a room to send secrets to"),
        (Command::JoinRoom, "Join a room to receive secrets from"),
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

pub async fn cli_prompt(stdin_tx: UnboundedSender<Message>) {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin);

    loop {
        let mut input = String::new();
        let _ = reader.read_line(&mut input).await;

        let command = match Command::from_string(&input.trim().split(' ').next().unwrap()) {
            Some(cmd) => cmd,
            None => {
                println!("\x1b[1m\x1b[95m\nUnknown command: {}\x1b[0m", input.trim());
                input.clear();
                Command::Help
            }
        };

        // TODO: replace with match
        if command.to_string() == Command::Help.to_string() {
            help_command();
        } else if command.to_string() == Command::Send.to_string() {
            // remove the command from the input and the new line at the end
            let slice = &input[Command::Send.to_string().len() + 1..input.len() - 1];
            stdin_tx.unbounded_send(Message::text(slice)).unwrap();
        } else if command.to_string() == Command::Quit.to_string() {
            break;
        } else {
            help_command();
        }
        input.clear();
    }
}
