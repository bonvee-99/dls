use url::Url;
use tungstenite::{connect, Message};

fn main () {
    let (mut socket, response) = connect(
        Url::parse("ws://localhost:3000").unwrap()
    ).expect("Can't connect");

    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
}
