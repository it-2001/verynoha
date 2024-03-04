use std::io::{Read, Write};

use common::connection_protocol::PlayerStatus;

pub struct ServerState {
    pub users: Users,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            users: Users::load_db(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UsersInfo {
    pub username: String,
    pub password: Vec<u8>,
    pub player_id: u64,
    pub friends: Vec<u64>,
    pub quote: String,
    pub funds: u64,
    pub status: PlayerStatus,
    pub card_collection: Vec<(u64, u8)>,
}

impl UsersInfo {
    fn new(username: String, password: Vec<u8>, id: u64) -> Self {
        Self {
            username,
            password,
            player_id: id,
            friends: Vec::new(),
            quote: String::new(),
            funds: 0,
            status: PlayerStatus::Offline,
            card_collection: Vec::new(),
        }
    }
}

pub struct Users {
    pub logins: Vec<UsersInfo>,
}

impl Users {
    pub fn new() -> Self {
        Self {
            logins: Vec::new(),
        }
    }

    pub fn add_login(&mut self, name: String, password: Vec<u8>) -> bool {
        if self.logins.iter().any(|login| login.username == name) {
            return false;
        }
        let id = self.logins.iter().map(|login| login.player_id).max().unwrap_or(0) + 1;
        let login = UsersInfo::new(name, password, id);
        self.logins.push(login);
        self.save_db();
        true
    }

    pub fn validate(&self, username: &str, password: &[u8]) -> Option<u64> {
        for login in &self.logins {
            if login.username == username && login.password == password {
                return Some(login.player_id);
            }
        }
        None
    }

    pub fn get_id(&self, username: &str) -> Option<u64> {
        for login in &self.logins {
            if login.username == username {
                return Some(login.player_id);
            }
        }
        None
    }

    pub fn get_username(&self, id: u64) -> Option<String> {
        for login in &self.logins {
            if login.player_id == id {
                return Some(login.username.clone());
            }
        }
        None
    }

    pub fn get(&self, id: u64) -> Option<&UsersInfo> {
        self.logins.iter().find(|login| login.player_id == id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut UsersInfo> {
        self.logins.iter_mut().find(|login| login.player_id == id)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut logins = Vec::new();
        let mut bytes = bytes;
        loop {
            if bytes.len() == 0 {
                break;
            }
            let mut reader = common::connection_protocol::ConnectionReader::new(common::connection_protocol::DB_USER, &bytes);
            println!("bytes: {:?}", bytes.len());
            let username = reader.read_string();
            let password = reader.read_binary();
            let id = reader.read_uint();
            let mut friends = Vec::new();
            let chunks = reader.read_binary();
            println!("chunks: {:?}", chunks.len());
            for chunk in chunks.chunks(8) {
                friends.push(u64::from_be_bytes(chunk[0..8].try_into().unwrap()));
            }
            let quote = reader.read_string();
            let funds = reader.read_uint();
            let status = PlayerStatus::Offline;
            let mut card_collection = Vec::new();
            for chunk in reader.read_binary().chunks(9) {
                let mut reader = common::connection_protocol::ConnectionReader::new(common::connection_protocol::DB_OWNED_CARD, chunk);
                let card_id = reader.read_uint();
                let amount = reader.read_uint() as u8;
                card_collection.push((card_id, amount));
            }
            logins.push(UsersInfo {
                username,
                password,
                player_id: id,
                friends,
                quote,
                funds,
                status,
                card_collection,
            });
            bytes = &bytes[reader.current_byte..];
        }
        Self {
            logins,
        }
    }

    pub fn load_db() -> Self {
        let mut file = std::fs::File::open("../db/users.txt").unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        Self::from_bytes(&contents)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        for login in &self.logins {
            let mut writer = common::connection_protocol::ConnectionWriter::new(common::connection_protocol::DB_USER);
            writer.write_string(&login.username);
            writer.write_binary(&login.password);
            writer.write_uint(login.player_id);
            let mut friends = Vec::new();
            for friend in &login.friends {
                friends.extend_from_slice(&friend.to_be_bytes());
            }
            writer.write_binary(&friends);
            writer.write_string(&login.quote);
            writer.write_uint(login.funds);
            let mut card_collection = Vec::new();
            for (card_id, amount) in &login.card_collection {
                let mut writer = common::connection_protocol::ConnectionWriter::new(common::connection_protocol::DB_OWNED_CARD);
                writer.write_uint(*card_id);
                writer.write_uint(*amount as u64);
                card_collection.extend(writer.finalize());
            }
            writer.write_binary(&card_collection);
            buffer.extend(writer.finalize());
        }
        buffer
    }

    pub fn save_db(&self) {
        let mut file = std::fs::File::create("../db/users.txt").unwrap();
        file.write_all(&self.to_bytes()).unwrap();
    }
}

