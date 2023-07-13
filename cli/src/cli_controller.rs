// use std::net::TcpStream;
use std::thread::spawn;
use tungstenite::{connect, Message};

const SERVER_URL: &str = "ws://localhost:8999";

// TODO: implement a way to send messages to the server
// TODO: integrate the cli input with how client sends and receives messages

enum Command {
    Start,
    CreateRoom,
    JoinRoom,
    Send,
    List,
    Help,
    Quit,
}

impl Command {
    fn from_string(s: &str) -> Option<Self> {
        match s {
            "start" => Some(Command::Start),
            "create room" => Some(Command::CreateRoom),
            "join room" => Some(Command::JoinRoom),
            "send" => Some(Command::Send),
            "list" => Some(Command::List),
            "help" => Some(Command::Help),
            "quit" => Some(Command::Quit),
            _ => None,
        }
    }

    fn to_string(&self) -> &'static str {
        match *self {
            Command::Start => "start",
            Command::CreateRoom => "create room",
            Command::JoinRoom => "join room",
            Command::Send => "send",
            Command::List => "list",
            Command::Help => "help",
            Command::Quit => "quit",
        }
    }
}

pub struct CliHandler {
}

impl CliHandler {
    pub fn new() -> Self {
        CliHandler {
        }
    }
    pub fn listen_to_server() {
        log_info("Connecting to server...");
        let (mut socket, _) = connect(SERVER_URL).expect("Failed to connect");
        // Receive messages from the server
        log_info("Session Created!");
        loop {
            match socket.read_message() {
                Ok(message) => {
                    match message {
                        Message::Text(text) => socket_message(&text),
                        _ => (),
                    }
                }
                Err(err) => {
                    eprintln!("Failed to read message: {}", err);
                    break;
                }
            }
        }
    }



    // pub async fn server_connection(& self) {
    //     // log_info("Connecting to server...");
    //     // let (socket, _) = connect(SERVER_URL).expect("Failed to connect");
    //     // self.socket = Some(socket);
    //     //
    //     // let socket = self.socket.as_mut().unwrap();
    //     // // Send a message to the server
    //     // log_info("Session Created!");
    //     // socket
    //     //     .write_message(Message::Text("Hello, server!".into()))
    //     //     .expect("Failed to send message");
    //     //
    //
    // }

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

    pub fn start_command() {
        println!("Starting a session...");
        // self.client.server_connection();

    }

    pub fn start(& self) {
          spawn(move || {
            CliHandler::listen_to_server();
        });

        loop {
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let command = match Command::from_string(&input.trim()) {
                Some(cmd) => cmd,
                None => {
                    println!("\x1b[1m\x1b[95mUnknown command: {}\x1b[0m", input.trim());
                    input.clear();
                    Command::Help
                }
            };

            match command {
                Command::Start => CliHandler::start_command(),
                Command::Quit => {
                    println!("Quitting...");
                    break;
                },
                Command::Help => CliHandler::help_command(),
                _ => CliHandler::help_command(),
            }

            input.clear();
        }

        // socket_conn.join().unwrap();
    }
}

pub fn log_info(message: &str) {
    println!("\x1b[1m\x1b[37mInfo: {}\x1b[0m", message);
}

pub fn socket_message(message: &str) {
    println!("\x1b[1m\x1b[95m>>\x1b[0m");
    println!("\x1b[1m\x1b[92m{}\x1b[0m", message);
    println!("\x1b[1m\x1b[95m<<\x1b[0m");

}
