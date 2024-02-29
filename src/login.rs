use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::task;

pub(crate) async fn login_to_server() -> Result<TcpStream, Box<dyn std::error::Error>> {
    println!("-- Enter login details --");
    print!("Username: ");
    let username = termui::input();
    print!("Password: ");
    // change this to a password input
    let password = termui::input();

    let addr = "127.0.0.1:3000".parse::<SocketAddr>()?;
    let socket = TcpSocket::new_v4()?;
    let mut stream = socket.connect(addr).await?;
    stream.write_all(username.as_bytes()).await?;
    stream.write_all(password.as_bytes()).await?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = std::str::from_utf8(&buffer[0..n])?;
    if response == "OK" {
        Ok(stream)
    } else {
        Err(format!("Server response: {response}").into())
    }
}