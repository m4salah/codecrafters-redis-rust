// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0; 1024];
                stream.read(&mut buf)?;
                let buf_string = String::from_utf8_lossy(&buf);
                let mut splited = buf_string.split("\r\n");

                if splited.next().unwrap() == "*1"
                    && splited.next().unwrap() == "$4"
                    && splited.next().unwrap().to_lowercase() == "ping"
                {
                    stream.write_all("+PONG\r\n".as_bytes())?;
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
