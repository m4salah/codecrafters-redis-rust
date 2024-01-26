// Uncomment this block to pass the first stage
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use clap::Parser;

struct Database {
    db: HashMap<String, DatabaseValue>,
    config: HashMap<String, String>,
}

struct DatabaseValue {
    time: Option<Time>,
    value: String,
}

struct Time {
    expires_at: SystemTime,
}

impl Database {
    fn new() -> Database {
        Database {
            db: HashMap::new(),
            config: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&String> {
        if let Some(value) = self.db.get(key) {
            if let Some(time) = &value.time {
                let now = SystemTime::now();
                if time.expires_at > now {
                    return Some(&value.value);
                }
            } else {
                return Some(&value.value);
            }
        }
        None
    }

    fn set(&mut self, key: &str, value: &str, expiry_ms: Option<u64>) -> Option<DatabaseValue> {
        if let Some(ms) = expiry_ms {
            let now = SystemTime::now();
            let expiry_duration = Duration::from_millis(ms);
            let expires_at = now + expiry_duration;
            let value = DatabaseValue {
                time: Some(Time { expires_at }),
                value: value.to_string(),
            };
            return self.db.insert(key.to_owned(), value);
        }
        let value = DatabaseValue {
            time: None,
            value: value.to_string(),
        };
        self.db.insert(key.to_owned(), value)
    }

    fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    fn set_config(&mut self, key: &str, value: &str) -> Option<String> {
        self.config.insert(key.to_owned(), value.to_string())
    }
}
#[derive(Parser, Debug)]
struct Args {
    /// Sets a custom config file
    #[arg(long)]
    dir: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(long)]
    dbfilename: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    let state = Arc::new(Mutex::new(Database::new()));

    if let Some(dir) = args.dir {
        state
            .lock()
            .unwrap()
            .set_config("dir", dir.to_str().unwrap());
    }
    if let Some(dbfilename) = args.dbfilename {
        state
            .lock()
            .unwrap()
            .set_config("dbfilename", dbfilename.to_str().unwrap());
    }
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
            match commands[0].to_lowercase().as_str() {
                "echo" => {
                    stream.write_all(format!("+{}\r\n", commands[1]).as_bytes())?;
                }
                "set" => {
                    if let Some(px_position) =
                        commands.iter().position(|v| v.to_lowercase() == "px")
                    {
                        let ms: u64 = commands[px_position + 1].parse().unwrap();

                        // lock on the state and set the value
                        {
                            state
                                .lock()
                                .unwrap()
                                .set(&commands[1], &commands[2], Some(ms));
                        }
                        stream.write_all(format!("+OK\r\n").as_bytes())?;
                    } else {
                        {
                            state.lock().unwrap().set(&commands[1], &commands[2], None);
                        }
                        stream.write_all(format!("+OK\r\n").as_bytes())?;
                    }
                }
                "get" => {
                    if let Some(v) = state.lock().unwrap().get(&commands[1]) {
                        stream.write_all(format!("+{v}\r\n").as_bytes())?;
                    } else {
                        stream.write_all(format!("$-1\r\n").as_bytes())?;
                    }
                }
                "config" => {
                    if commands[1].to_lowercase() == "get" {
                        if let Some(v) = state.lock().unwrap().get_config(commands[2]) {
                            stream.write_all(
                                format!(
                                    "*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                                    commands[2].len(),
                                    commands[2],
                                    v.len(),
                                    v
                                )
                                .as_bytes(),
                            )?;
                        } else {
                            stream.write_all(format!("$-1\r\n").as_bytes())?;
                        }
                    }
                }
                _ => {
                    stream.write_all("+PONG\r\n".as_bytes())?;
                }
            }
        } else {
            stream.write_all("+PONG\r\n".as_bytes())?;
        }
    }
}
