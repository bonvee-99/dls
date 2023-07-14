//! A simple example of hooking up stdin/stdout to a WebSocket stream.
//!
//! This example will connect to a server specified in the argument list and
//! then forward all data read on stdin to the server, printing out all data
//! received on stdout.
//!
//! Note that this is not currently optimized for performance, especially around
//! buffer management. Rather it's intended to show an example of working with a
//! client.
//!
//! You can use this example together with the `server` example.

// use std::env;

use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_channel::mpsc;

#[tokio::main]
async fn main() {
    let connect_addr = "ws://localhost:3000";
    let url = url::Url::parse(&connect_addr).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to websocket");

    let (stdin_tx, stdin_rx) = mpsc::unbounded::<Message>();
    
    tokio::spawn(read_stdin(stdin_tx));

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            // let data2 = data.clone();
            // println!("got message: {:?}", String::from_utf8(data2).unwrap());
            tokio::io::stdout().write_all(&data).await.unwrap();
            tokio::io::stdout().flush().await.unwrap();
            // std::io::stdout().write_all(&data2).unwrap();
            // std::io::stdout().flush().unwrap();
        })
    };
    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
