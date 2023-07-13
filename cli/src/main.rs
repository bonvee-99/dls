mod cli_controller;
use std::thread::spawn;

// entry point
fn main() {
    println!("DLS CLI v0.1.0");

    let cli_thread = spawn( || {
        println!("Waiting for message...");
        let cli_handler = cli_controller::CliHandler::new();
        cli_handler.start();
    });

    // prevent closing if other thread is active
    cli_thread.join().unwrap();
    println!("Thank you for using DLS CLI!");
}



