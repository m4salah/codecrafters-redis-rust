// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream) {
                        eprintln!("ERROR: while handling the connection: {e}");
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut buf = [0; 1024];
    loop {
        let n = stream.read(&mut buf)?;
        let buf_str = String::from_utf8_lossy(&buf[..n]);
        eprintln!("{}", buf_str);
        let splitted = buf_str.split("\r\n");
        let mut v = Vec::new();
        for command in splitted {
            if !command.starts_with("*") && !command.starts_with("$") && command != "" {
                eprintln!("{:?}", command);
                v.push(command);
            }
        }
        if !v.is_empty() {
            if v[0].to_lowercase() == "echo" {
                stream.write_all(format!("+{}\r\n", v[1]).as_bytes())?;
                continue;
            }
        }
        eprintln!("{:?}", v);
        stream.write_all("+PONG\r\n".as_bytes())?;
    }
}
