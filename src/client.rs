use tokio::net::TcpStream;

use termui::*;

pub async fn start_client(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    println!("Connected to server");
    let data = match common::connection_protocol::Message::read_stream(stream).await {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to read response: {e:?}");
            return Err("Failed to read response".into());
        }
    };
    println!("Received: {data:?}");

    wait();
    Ok(())
}