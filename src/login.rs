use std::net::SocketAddr;
use termui::wait_clear;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpSocket, TcpStream};

fn ask_for_login() -> Option<(String, String)> {
    loop {
        print!("Username: ");
        let username = termui::input();
        print!("Password: ");
        let password = termui::input();
        if username.is_empty() || password.is_empty() {
            return None;
        }
        let login = common::login::validate(&username, &password);
        if login.is_valid() {
            return Some((username, password));
        }
        println!("Invalid login: {login}");
    }
}

async fn make_conncetion() -> Result<TcpStream, Box<dyn std::error::Error>> {
    let addr = common::DEFAULT_SERVER_IP;
    let stream = match connect(Some(addr)).await {
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
    Ok(stream)
}

pub async fn login_to_server() -> Result<TcpStream, Box<dyn std::error::Error>> {
    let (username, password) = match ask_for_login() {
        Some((username, password)) => (username, password),
        None => return Err("Login cancelled".into()),
    };
    
    let hash = common::login::hash_password(&password);
    
    let mut stream = make_conncetion().await?;

    let buffer = common::connection_protocol::Message::Login { username: username.clone(), password: hash }.to_bytes();
    println!("sending buffer: {:?}", buffer.len());
    stream.write(&buffer).await?;

    let response = match common::connection_protocol::Message::read_stream(&mut stream).await {
        Ok(response) => response,
        Err(e) => {
            println!("Failed to read response: {e:?}");
            return Err("Failed to read response".into());
        }
    };

    let res = match response {
        common::connection_protocol::Message::Ok(_, _) => {
            println!("Login successful");
            Ok(stream)
        }
        common::connection_protocol::Message::Error(_, Some(message)) => {
            println!("Login failed: {}", String::from_utf8_lossy(&message));
            Err("Login failed".into())
        }
        common::connection_protocol::Message::Error(_, None) => {
            println!("Login failed");
            Err("Login failed".into())
        }
        _ => {
            Err("Unexpected response".into())
        }
    };
    wait_clear();
    res
}

pub async fn register() -> Result<TcpStream, Box<dyn std::error::Error>> {
    let (username, password) = match ask_for_login() {
        Some((username, password)) => (username, password),
        None => return Err("Registration cancelled".into()),
    };
    
    let hash = common::login::hash_password(&password);
    
    let mut stream = make_conncetion().await?;

    let buffer = common::connection_protocol::Message::Register { username: username.clone(), password: hash }.to_bytes();
    
    stream.write(&buffer).await?;

    let response = match common::connection_protocol::Message::read_stream(&mut stream).await {
        Ok(response) => response,
        Err(e) => {
            println!("Failed to read response: {e:?}");
            return Err("Failed to read response".into());
        }
    };

    let res = match response {
        common::connection_protocol::Message::Ok(_, _) => {
            println!("Registration successful");
            Ok(stream)
        }
        common::connection_protocol::Message::Error(_, Some(message)) => {
            println!("Registration failed: {}", String::from_utf8_lossy(&message));
            Err("Registration failed".into())
        }
        common::connection_protocol::Message::Error(_, None) => {
            println!("Registration failed");
            Err("Registration failed".into())
        }
        _ => {
            Err("Unexpected response".into())
        }
    };
    wait_clear();
    res
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
