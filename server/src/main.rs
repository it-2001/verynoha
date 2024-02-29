/// rust tcp multi-threaded server
use std::sync::{Arc, Mutex};
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
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
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
                }).await.unwrap();
                socket.write_all(b"OK").await.unwrap();
            }
        });
    }
}

