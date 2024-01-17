// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    loop {
        match listener.accept() {
            Ok((mut stream, addr)) => {
                println!("new client with address: {addr} connected");
                thread::spawn(move || {
                    let mut buf = [0; 1024];
                    stream.read(&mut buf).unwrap();
                    stream.write_all("+PONG\r\n".as_bytes()).unwrap();
                });
            }
            Err(e) => println!("Couldn't get client: {:?}", e),
        }
    }
}
