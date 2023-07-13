use std::io::{self, Write};
use tungstenite::{connect, Message};
use url::Url;

fn main() {
    // Read the WebSocket server URL from the command line
    // let mut server_url = String::new();
    // print!("Enter the WebSocket server URL: ");
    // io::stdout().flush().unwrap();
    // io::stdin().read_line(&mut server_url).unwrap();
    // let server_url = server_url.trim();

    // Connect to the WebSocket server
    let (mut socket, _) = connect("ws://localhost:3000").expect("Failed to connect");

    // Read the message to send from the command line
    // let mut message = String::new();
    // print!("Enter the message to send: ");
    // io::stdout().flush().unwrap();
    // io::stdin().read_line(&mut message).unwrap();
    //
    // // Send the message to the server
    // socket.write_message(Message::Text(message.into())).expect("Failed to send message");
    //
    // Receive messages from the server
    
    // TODO: we need to split / share the socket so we can read from the cli in the main thread and read from the socket in a separate thread
    loop {
        let message = socket.read_message().expect("Failed to read message");
        match message {
            Message::Text(text) => println!("Received message: {}", text),
            Message::Binary(_) => println!("Received binary message"),
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) => (),
        }
    }
}


/* TODO:
- handle transforming into JSON to send to server
- handle parsing JSON
*/
