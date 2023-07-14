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
    let ( write, read )= server_connection().await;

    let stdin_to_ws = tokio::spawn(write_text_to_server(write, stdin_rx));
    let listen_stream = tokio::spawn(listen_to_server(read));

    let cli_sender = stdin_tx.clone();
    tokio::spawn(cli_prompt(cli_sender));
    future::select(stdin_to_ws, listen_stream).await;
}

// establish connection to server
// and create a stream to listen to server and a stream to write to server
async fn server_connection() -> (SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    let connection_url = "ws://localhost:3000";
    let url = url::Url::parse(&connection_url).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    util::log_info("Connected to server");
    return ws_stream.split();
}

async fn write_text_to_server(mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, mut stdin_rx: UnboundedReceiver<Message>) {
    loop {
        let msg = stdin_rx.next().await.unwrap();
        // println!("write_text_to_server: {:?}", msg);
        write.send(msg).await.unwrap();
    }
}

 async fn listen_to_server(read:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>)  {
      read.for_each(|message| async {
         let data = message.unwrap().into_data();
          println!("\x1b[1m\x1b[95m>>\x1b[0m");
          println!("\x1b[1m\x1b[36m{:?}\x1b[0m", String::from_utf8(data).unwrap());
          println!("\x1b[1m\x1b[95m<<\x1b[0m");
         // let data2 = data.clone();
         // println!("got message: {:?}", String::from_utf8(data2).unwrap());
         // tokio::io::stdout().write_all(&data).await.unwrap();
          // std::io::stdout().write_all(&data2).unwrap();
          // std::io::stdout().flush().unwrap();
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

    pub async fn start() {

        let (stdin_tx, stdin_rx) = mpsc::unbounded::<Message>();
        let ( write, read )= server_connection().await;

        let stdin_to_ws = tokio::spawn(write_text_to_server(write, stdin_rx));
        let listen_stream = tokio::spawn(listen_to_server(read));
        let stdin_tx2 = stdin_tx.clone();
        tokio::spawn(cli_prompt(stdin_tx2));
        stdin_tx.unbounded_send(Message::text("HHHH")).unwrap();

        future::select(stdin_to_ws, listen_stream).await;
    }

pub async fn cli_prompt(stdin_tx: UnboundedSender<Message>) {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin);

    loop {
        let mut input = String::new();
        // Read input asynchronously
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


// TODO: once cli global controls such as ^C are implemented, this will be the main sender
// read from stdin and request to send to server
// async fn write_to_server(tx: mpsc::UnboundedSender<Message>) {
//     let mut stdin = tokio::io::stdin();
//     loop {
//         let mut buf = vec![0; 1024];
//         let n = match stdin.read(&mut buf).await {
//             Err(_) | Ok(0) => break,
//             Ok(n) => n,
//         };
//         buf.truncate(n);
//         tx.unbounded_send(Message::binary(buf)).unwrap();
//     }
// }


