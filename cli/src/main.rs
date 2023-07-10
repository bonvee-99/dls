use tungstenite::{connect, Message};

fn main() {
    // Connect to the WebSocket server
    let (mut socket, _) = connect("ws://localhost:8999").expect("Failed to connect");

    // Send a message to the server
    socket.write_message(Message::Text("Hello, server!".into())).expect("Failed to send message");

    // Receive messages from the server
    loop {
        let message = socket.read_message().expect("Failed to read message");
        match message {
            Message::Text(text) => println!("Received message: {}", text),
            Message::Binary(_) => println!("Received binary message"),
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) => (),
        }
    }
}
