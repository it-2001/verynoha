use termui::*;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::task;

mod login;


#[tokio::main]
async fn main() {
    let connection = loop {
        match login::login_to_server().await {
            Ok(connection) => break connection,
            Err(e) => println!("Error: {e}"),
        }
    };
    
}
