use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::task;

pub(crate) async fn login_to_server() -> Result<TcpStream, Box<dyn std::error::Error>> {
    println!("-- Enter login details --");
    print!("Username: ");
    let username = termui::input();
    print!("Password: ");
    // change this to a password input
    let password = termui::input();

    let login = common::login::validate(&username, &password);
    if !login.is_valid() {
        return Err(format!("Invalid login: {login}").into());
    }

    let addr = "127.0.0.1:3000";
    let mut stream = match connect(Some(addr)).await {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to connect to server: {e}");
            let mut tcp_stream = None;
            for _ in 0..3 {
                println!("Retrying in 3 seconds...");
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                match connect(Some(addr)).await {
                    Ok(stream) => {
                        tcp_stream = Some(stream);
                        break;
                    }
                    Err(_) => (),
                }
            }
            match tcp_stream {
                Some(stream) => stream,
                None => return Err("Failed to connect to server\nIf you want to specify a different server, please look into the manual".into()),
            }
        }
    };
    let buffer =
        common::connection_protocol::ConnectionWriter::new(common::connection_protocol::LOGIN)
            .write_string(&username)
            .write_string(&password)
            .finalize();
    
    stream.write_all(&buffer).await?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = std::str::from_utf8(&buffer[0..n])?;
    if response == "OK" {
        Ok(stream)
    } else {
        Err(format!("Server response: {response}").into())
    }
}

async fn connect(addr: Option<&str>) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let addr = match addr {
        Some(addr) => addr.parse::<SocketAddr>()?,
        None => {
            print!("Manually enter the server address: ");
            let addr = termui::input();
            addr.parse::<SocketAddr>()?
        }
    };
    let socket = TcpSocket::new_v4()?;
    let stream = socket.connect(addr).await?;
    Ok(stream)
}
