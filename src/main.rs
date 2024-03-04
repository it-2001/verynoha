use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use termui::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::task;

mod login;
mod options;
mod client;

#[tokio::main]
async fn main() {
    'main: loop {
        termui::clear_screen();
        let op = options(&["Login", "Register", "Options", "Exit"]);
        termui::clear_screen();
        match op {
            0 => {
                let mut connection = match login::login_to_server().await {
                    Ok(connection) => connection,
                    Err(e) =>  {
                        println!("Failed to connect to server: {e}");
                        continue;
                    },
                };
                match client::start_client(&mut connection).await {
                    Ok(_) => (),
                    Err(e) => println!("Failed to start client: {e}"),
                }
            },
            1 => {
                let mut connection = match login::register().await {
                    Ok(connection) => connection,
                    Err(e) => {
                        println!("Failed to connect to server: {e}");
                        continue;
                    },
                };
                match client::start_client(&mut connection).await {
                    Ok(_) => (),
                    Err(e) => println!("Failed to start client: {e}"),
                }
            }
            //2 => options(),
            3 => break 'main,
            _ => (),
        }
    }
    println!("Goodbye!");
}
