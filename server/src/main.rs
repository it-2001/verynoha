/// rust tcp multi-threaded server
use std::sync::{Arc, Mutex};
use common::connection_protocol::LOGIN;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task;
use tokio::net::*;
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let shared_data = Arc::new(Mutex::new(0));
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let shared_data = Arc::clone(&shared_data);
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let n = match socket.read(&mut buffer).await {
                    Ok(n) => {
                        if n == 0 {
                            return;
                        }
                        n
                    }
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {e}");
                        return;
                    }
                };
                let shared_data = Arc::clone(&shared_data);
                task::spawn_blocking(move || {
                    let mut data = shared_data.lock().unwrap();
                    *data += 1;
                    println!("Data: {data}");
                    let mut reader = common::connection_protocol::ConnectionReader::new(LOGIN, &buffer[0..n]);
                    let username = reader.read_string();
                    let password = reader.read_string();
                    let login = common::login::validate(&username, &password);
                    if login.is_valid() {
                        println!("Login successful: {username}\nPassword: {password}");
                    } else {
                        println!("Login failed: {username}");
                    }
                }).await.unwrap();
                socket.write_all(b"OK").await.unwrap();
            }
        });
    }
}

