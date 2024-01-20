// Uncomment this block to pass the first stage
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

struct Database {
    db: HashMap<String, String>,
}

impl Database {
    fn new() -> Database {
        Database { db: HashMap::new() }
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.db.get(key)
    }

    fn set(&mut self, key: &str, value: &str) -> Option<String> {
        self.db.insert(key.to_owned(), value.to_owned())
    }
}

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    let state = Arc::new(Mutex::new(Database::new()));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = state.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, state) {
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

fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<Database>>) -> anyhow::Result<()> {
    let mut buf = [0; 1024];
    loop {
        let n = stream.read(&mut buf)?;
        let buf_str = String::from_utf8_lossy(&buf[..n]);
        eprintln!("{}", buf_str);
        let splitted = buf_str.split("\r\n");

        let mut commands = Vec::new();
        for command in splitted {
            if !command.starts_with("*") && !command.starts_with("$") && command != "" {
                eprintln!("{:?}", command);
                commands.push(command);
            }
        }
        if !commands.is_empty() {
            if commands[0].to_lowercase() == "echo" {
                stream.write_all(format!("+{}\r\n", commands[1]).as_bytes())?;
                continue;
            }
            if commands[0].to_lowercase() == "set" {
                {
                    state.lock().unwrap().set(&commands[1], &commands[2]);
                }
                stream.write_all(format!("+OK\r\n").as_bytes())?;
                continue;
            }
            if commands[0].to_lowercase() == "get" {
                if let Some(v) = state.lock().unwrap().get(&commands[1]) {
                    stream.write_all(format!("+{v}\r\n").as_bytes())?;
                } else {
                    stream.write_all(format!("$-1\r\n").as_bytes())?;
                }
                continue;
            }
        }
        eprintln!("{:?}", commands);
        stream.write_all("+PONG\r\n".as_bytes())?;
    }
}
