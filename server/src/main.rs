//! rust tcp multi-threaded server
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::*;

mod db;

#[tokio::main]
async fn main() {
    let mut state = db::ServerState::new();
    state
        .users
        .add_login("admin".to_string(), common::login::hash_password("admin"));
    state
        .users
        .add_login("user".to_string(), common::login::hash_password("user"));
    state
        .users
        .add_login("test".to_string(), common::login::hash_password("test"));

    println!("users: {:?}", state.users.logins.iter().map(|u|u.username.clone()).collect::<Vec<String>>());
    let state = Arc::new(Mutex::new(state));

    let listener = TcpListener::bind(common::DEFAULT_SERVER_IP).await.unwrap();

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            handle_client(socket, state).await.unwrap();
        });
    }
}

async fn handle_client(
    mut socket: TcpStream,
    state: Arc<Mutex<db::ServerState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let msg = common::connection_protocol::Message::read_stream(&mut socket).await;
        match msg {
            Ok(common::connection_protocol::Message::Login { username, password }) => {
                let id = {
                    let state = state.lock().unwrap();
                    state.users.validate(&username, &password)
                };
                if let Some(id) = id {
                    println!("User {} logged in", username);
                    let buffer = common::connection_protocol::Message::Ok(0, None).to_bytes();
                    socket.write(&buffer).await?;
                    let player_data = {
                        let mut state = state.lock().unwrap();
                        let usr = state.users.get_mut(id).unwrap();
                        usr.status = common::connection_protocol::PlayerStatus::Online;
                        let usr = state.users.get(id).unwrap();
                        let mut friends = Vec::new();
                        for friend_id in &usr.friends {
                            let friend = state.users.get(*friend_id).unwrap();
                            let friend = common::connection_protocol::Friend {
                                username: friend.username.clone(),
                                status: friend.status.clone(),
                                quote: friend.quote.clone(),
                            };
                            friends.push(friend);
                        }
                        common::connection_protocol::ClientData {
                            username: usr.username.clone(),
                            funds: usr.funds,
                            quote: usr.quote.clone(),
                            friends,
                            status: usr.status.clone(),
                        }
                    };
                    let buff = common::connection_protocol::Message::ClientData(player_data).to_bytes();
                    println!("Sending: {:?}", buff.len());
                    socket.write(&buff).await?;
                    return Ok(());
                }
                let buffer = common::connection_protocol::Message::Error(
                    0,
                    Some("Invalid login".as_bytes().to_vec()),
                )
                .to_bytes();
                socket.write(&buffer).await?;
                return Ok(());
            }
            Ok(common::connection_protocol::Message::Register { username, password }) => {
                if state
                    .lock()
                    .unwrap()
                    .users
                    .add_login(username, password)
                {
                    let buffer = common::connection_protocol::Message::Ok(0, None).to_bytes();
                    socket.write(&buffer).await?;
                    return Ok(());
                }
                let buffer = common::connection_protocol::Message::Error(
                    0,
                    Some("User already exists".as_bytes().to_vec()),
                )
                .to_bytes();
                socket.write(&buffer).await?;
                return Ok(());
            }
            _ => (),
        }
    }
}
