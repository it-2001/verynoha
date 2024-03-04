use connection_protocol::FriendSummary;

pub const DEFAULT_SERVER_IP: &str = "127.0.0.1:3000";

/// helper module for login validation
pub mod login {
    pub const USERNAME_MIN: usize = 3;
    pub const USERNAME_MAX: usize = 20;
    pub const PASSWORD_MIN: usize = 5;
    pub const PASSWORD_MAX: usize = 20;

    /// Validate a username and password
    ///
    /// Username
    /// - Must be at least 3 characters
    /// - Must be at most 20 characters
    /// - Unicode is allowed
    ///
    /// Password
    /// - Must be at least 5 characters (no one is going to hack this anyway copium)
    /// - Must be at most 20 characters
    /// - Unicode is allowed
    ///
    /// Login also can not contain any whitespace
    pub fn validate(username: &str, password: &str) -> LoginValidation {
        let u_chars = username.chars().count();
        let p_chars = password.chars().count();

        if u_chars < USERNAME_MIN {
            return LoginValidation::UsernameTooShort;
        }
        if u_chars > USERNAME_MAX {
            return LoginValidation::UsernameTooLong;
        }
        if p_chars < PASSWORD_MIN {
            return LoginValidation::PasswordTooShort;
        }
        if p_chars > PASSWORD_MAX {
            return LoginValidation::PasswordTooLong;
        }
        if username.chars().any(|c| c.is_whitespace()) {
            return LoginValidation::UsernameContainsWhitespace;
        }
        if password.chars().any(|c| c.is_whitespace()) {
            return LoginValidation::PasswordContainsWhitespace;
        }
        LoginValidation::Valid
    }

    /// The result of a login validation
    #[derive(Debug, PartialEq)]
    pub enum LoginValidation {
        UsernameTooShort,
        UsernameTooLong,
        UsernameContainsWhitespace,
        PasswordTooShort,
        PasswordTooLong,
        PasswordContainsWhitespace,

        Valid,
    }

    impl std::fmt::Display for LoginValidation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                LoginValidation::UsernameTooShort => write!(f, "Username is too short"),
                LoginValidation::UsernameTooLong => write!(f, "Username is too long"),
                LoginValidation::UsernameContainsWhitespace => {
                    write!(f, "Username contains whitespace")
                }
                LoginValidation::PasswordTooShort => write!(f, "Password is too short"),
                LoginValidation::PasswordTooLong => write!(f, "Password is too long"),
                LoginValidation::PasswordContainsWhitespace => {
                    write!(f, "Password contains whitespace")
                }
                LoginValidation::Valid => write!(f, "Valid"),
            }
        }
    }

    impl LoginValidation {
        pub fn is_valid(&self) -> bool {
            match self {
                LoginValidation::Valid => true,
                _ => false,
            }
        }
    }

    pub fn hash_password(password: &str) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(password);
        hasher.finalize().to_vec()
    }
}

/// helper module for validating and defining the connection protocol
pub mod connection_protocol {
    use std::io::Read;

    use tokio::{io::AsyncReadExt, net::TcpStream};

    /// The default protocol for the connection
    pub const CONTAINER: &'static [Chunks] = &[
        // head
        Chunks::Uint { size: 8 },
        // body
        Chunks::Binary,
    ];

    /// The default protocol for the database user entry
    pub const DB_USER: &'static [Chunks] = &[
        // username
        Chunks::String,
        // password hash
        Chunks::Binary,
        // player id
        Chunks::Uint { size: 8 },
        // friends
        //
        // just an array of ids
        Chunks::Binary,
        // quote
        Chunks::String,
        // funds
        Chunks::Uint { size: 8 },
        // card collection
        //
        // just an array of (id, amount) tuples
        Chunks::Binary,
    ];

    /// The default protocol for the database card entry
    pub const DB_OWNED_CARD: &'static [Chunks] = &[
        // card id
        Chunks::Uint { size: 8 },
        // amount
        Chunks::Uint { size: 1 },
    ];

    /// Expands the clientdata struct into a protocol
    pub const CLIENT_DATA: &'static [Chunks] = &[
        // username
        Chunks::String,
        // friends
        Chunks::Binary,
        // funds
        Chunks::Uint { size: 8 },
        // status
        Chunks::Uint { size: 8 },
        // quote
        Chunks::String,
    ];

    /// The default protocol for the friend summary
    pub const FRIEND_SUMMARY: &'static [Chunks] = &[
        // username
        Chunks::String,
        // quote
        Chunks::String,
        // status
        Chunks::Uint { size: 1 },
    ];

    /// The default protocol for login and registration
    pub const LOGIN: &'static [Chunks] = &[
        // username
        Chunks::String,
        // password hash
        Chunks::Binary,
    ];

    /// The default protocol for saving the DB
    pub const LOAD: &'static [Chunks] = &[
        // username
        Chunks::String,
        // password
        Chunks::String,
        // auto login
        Chunks::Bool,
        // server ip
        Chunks::String,
    ];

    /// The default protocol for status messages
    pub const STATUS: &'static [Chunks] = &[
        // player id
        Chunks::Uint { size: 8 },
        // flag for message
        Chunks::Bool,
        // message
        Chunks::Binary,
    ];

    /// The default protocol for dynamic messages
    ///
    /// note: set mode to unchecked to get better performance and more predictable results
    pub const DYNAMIC: &'static [Chunks] = &[Chunks::Rest];

    /// Holds the connection protocol
    pub struct ConnectionProtocol {
        pub chunks: &'static [Chunks],
    }

    /// Holds the data neccesary to write a message to the connection
    pub struct ConnectionWriter {
        pub current_chunk: usize,
        pub msg: Vec<u8>,
        pub protocol: &'static [Chunks],
        pub unchecked: bool,
    }

    impl ConnectionWriter {
        /// Create a new connection writer
        pub fn new(protocol: &'static [Chunks]) -> ConnectionWriter {
            // this is only a wild guess
            let capacity = protocol.len() * 32;
            ConnectionWriter {
                current_chunk: 0,
                msg: Vec::with_capacity(capacity),
                protocol,
                unchecked: false,
            }
        }

        /// Set the writer to unchecked mode (will not check if all chunks were written or if the bytes are valid for the chunk type)
        pub fn unchecked(mut self) -> Self {
            self.unchecked = true;
            self
        }

        /// Write an integer to the message
        pub fn write_int(&mut self, value: i64) -> &mut Self {
            if self.unchecked {
                self.msg.extend_from_slice(&value.to_be_bytes());
                return self;
            }

            match &self.protocol[self.current_chunk] {
                Chunks::Int { size } => {
                    self.msg
                        .extend_from_slice(&value.to_be_bytes()[8 - *size as usize..]);
                    self.current_chunk += 1;
                }
                _ => panic!(
                    "Invalid chunk type, expected {:?}, got int",
                    self.protocol[self.current_chunk]
                ),
            }
            self
        }

        /// Write a float to the message
        pub fn write_float(&mut self, value: f64) -> &mut Self {
            if self.unchecked {
                self.msg.extend_from_slice(&value.to_be_bytes());
                return self;
            }

            match &self.protocol[self.current_chunk] {
                Chunks::Float => {
                    self.msg.extend_from_slice(&value.to_be_bytes());
                    self.current_chunk += 1;
                }
                _ => panic!(
                    "Invalid chunk type, expected {:?}, got float",
                    self.protocol[self.current_chunk]
                ),
            }
            self
        }

        /// Write an unsigned integer to the message
        pub fn write_uint(&mut self, value: u64) -> &mut Self {
            if self.unchecked {
                self.msg.extend_from_slice(&value.to_be_bytes());
                return self;
            }

            match &self.protocol[self.current_chunk] {
                Chunks::Uint { size } => {
                    self.msg
                        .extend_from_slice(&value.to_be_bytes()[8 - *size as usize..]);
                    self.current_chunk += 1;
                }
                _ => panic!(
                    "Invalid chunk type, expected {:?}, got uint\nExecute protocol: {:?}",
                    self.protocol[self.current_chunk], self.protocol
                ),
            }
            self
        }

        /// Write a string to the message
        pub fn write_string(&mut self, value: &str) -> &mut Self {
            if self.unchecked {
                let value = value.as_bytes();
                let size = value.len() as u64;
                self.msg.extend_from_slice(&size.to_be_bytes());
                self.msg.extend_from_slice(value);
                return self;
            }

            match &self.protocol[self.current_chunk] {
                Chunks::String => {
                    let value = value.as_bytes();
                    let size = value.len() as u64;
                    self.msg.extend_from_slice(&size.to_be_bytes());
                    self.msg.extend_from_slice(value);
                    self.current_chunk += 1;
                }
                _ => panic!(
                    "Invalid chunk type, expected {:?}, got string\nExecute protocol: {:?}",
                    self.protocol[self.current_chunk], self.protocol
                ),
            }
            self
        }

        /// Write a binary to the message
        pub fn write_binary(&mut self, value: &[u8]) -> &mut Self {
            if self.unchecked {
                let size = value.len() as u64;
                self.msg.extend_from_slice(&size.to_be_bytes());
                self.msg.extend_from_slice(value);
                return self;
            }

            match &self.protocol[self.current_chunk] {
                Chunks::Binary => {
                    let size = value.len() as u64;
                    self.msg.extend_from_slice(&size.to_be_bytes());
                    self.msg.extend_from_slice(value);
                    self.current_chunk += 1;
                }
                _ => panic!(
                    "Invalid chunk type, expected {:?}, got binary\nExecute protocol: {:?}",
                    self.protocol[self.current_chunk], self.protocol
                ),
            }
            self
        }

        /// Write a boolean to the message
        pub fn write_bool(&mut self, value: bool) -> &mut Self {
            if self.unchecked {
                self.msg.push(if value { 1 } else { 0 });
                return self;
            }

            match &self.protocol[self.current_chunk] {
                Chunks::Bool => {
                    self.msg.push(if value { 1 } else { 0 });
                    self.current_chunk += 1;
                }
                _ => panic!(
                    "Invalid chunk type, expected {:?}, got bool\nExecute protocol: {:?}",
                    self.protocol[self.current_chunk], self.protocol
                ),
            }
            self
        }

        /// Finish writing the message
        pub fn finalize(&mut self) -> Vec<u8> {
            if self.current_chunk != self.protocol.len() {
                panic!("Not all chunks were written");
            }
            self.msg.clone()
        }

        /// Finish without checking if all chunks were written
        pub fn finalize_unchecked(&mut self) -> Vec<u8> {
            self.msg.clone()
        }
    }

    /// Holds the data neccesary to read a message from the connection
    #[derive(Debug)]
    pub struct ConnectionReader<'a> {
        pub current_byte: usize,
        pub current_chunk: usize,
        pub msg: &'a [u8],
        pub protocol: &'static [Chunks],
        pub unchecked: bool,
    }

    impl<'a> ConnectionReader<'a> {
        /// Create a new connection reader
        pub fn new(protocol: &'static [Chunks], msg: &'a [u8]) -> ConnectionReader<'a> {
            ConnectionReader {
                current_byte: 0,
                current_chunk: 0,
                msg,
                protocol,
                unchecked: false,
            }
        }

        /// Set the reader to unchecked mode (will not check if all chunks were read or if the bytes are valid for the chunk type)
        pub fn unchecked(mut self) -> Self {
            self.unchecked = true;
            self
        }

        /// Read an integer from the message
        pub fn read_int(&mut self) -> i64 {
            if self.unchecked {
                let value = i64::from_be_bytes(
                    self.msg[self.current_byte..self.current_byte + 8]
                        .try_into()
                        .unwrap(),
                );
                self.current_byte += 8;
                println!("Int: {value}");
                return value;
            }
            match &self.protocol[self.current_chunk] {
                Chunks::Int { size } => {
                    let value = i64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + *size as usize]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_chunk += 1;
                    self.current_byte += *size as usize;
                    value
                }
                Chunks::Any | Chunks::Rest => {
                    let value = i64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + 8]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_byte += 8;
                    if let Chunks::Any = self.protocol[self.current_chunk] {
                        self.current_chunk += 1;
                    }
                    value
                }
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read a float from the message
        pub fn read_float(&mut self) -> f64 {
            if self.unchecked {
                let value = f64::from_be_bytes(
                    self.msg[self.current_byte..self.current_byte + 8]
                        .try_into()
                        .unwrap(),
                );
                self.current_byte += 8;
                println!("Float: {value}");
                return value;
            }
            match &self.protocol[self.current_chunk] {
                Chunks::Float => {
                    let value = f64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + 8]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_byte += 8;
                    self.current_chunk += 1;
                    value
                }
                Chunks::Any | Chunks::Rest => {
                    let value = f64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + 8]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_byte += 8;
                    if let Chunks::Any = self.protocol[self.current_chunk] {
                        self.current_chunk += 1;
                    }
                    value
                }
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read an unsigned integer from the message
        pub fn read_uint(&mut self) -> u64 {
            if self.unchecked {
                let value = u64::from_be_bytes(
                    self.msg[self.current_byte..self.current_byte + 8]
                        .try_into()
                        .unwrap(),
                );
                self.current_byte += 8;
                println!("Uint: {value}");
                return value;
            }
            match &self.protocol[self.current_chunk] {
                Chunks::Uint { size } => {
                    let value = u64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + *size as usize]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_byte += *size as usize;
                    self.current_chunk += 1;
                    value
                }
                Chunks::Any | Chunks::Rest => {
                    let value = u64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + 8]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_byte += 8;
                    if let Chunks::Any = self.protocol[self.current_chunk] {
                        self.current_chunk += 1;
                    }
                    value
                }
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read a string from the message
        pub fn read_string(&mut self) -> String {
            if self.unchecked {
                let size = u64::from_be_bytes(
                    self.msg[self.current_byte..self.current_byte + 8]
                        .try_into()
                        .unwrap(),
                );
                self.current_byte += 8;
                let value = String::from_utf8(
                    self.msg[self.current_byte..self.current_byte + size as usize].to_vec(),
                )
                .unwrap();
                self.current_byte += size as usize;
                return value;
            }
            match &self.protocol[self.current_chunk] {
                Chunks::String => {
                    println!("Current byte: {}", self.current_byte);
                    println!("Current chunk: {}", self.current_chunk);
                    let size = u64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + 8]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_byte += 8;
                    let value = String::from_utf8(
                        self.msg[self.current_byte..self.current_byte + size as usize].to_vec(),
                    )
                    .unwrap();
                    self.current_byte += size as usize;
                    self.current_chunk += 1;
                    value
                }
                Chunks::Any | Chunks::Rest => {
                    let value = String::from_utf8(
                        self.msg[self.current_byte..self.current_byte + 8].to_vec(),
                    )
                    .unwrap();
                    self.current_byte += 8;
                    if let Chunks::Any = self.protocol[self.current_chunk] {
                        self.current_chunk += 1;
                    }
                    value
                }
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read a binary from the message
        pub fn read_binary(&mut self) -> Vec<u8> {
            if self.unchecked {
                let size = u64::from_be_bytes(
                    self.msg[self.current_byte..self.current_byte + 8]
                        .try_into()
                        .unwrap(),
                );
                self.current_byte += 8;
                let value = self.msg[self.current_byte..self.current_byte + size as usize].to_vec();
                self.current_byte += size as usize;
                println!("Size: {size}");
                println!("Binary: {:x?}", value);
                return value;
            }
            match &self.protocol[self.current_chunk] {
                Chunks::Binary => {
                    println!("Current byte: {}", self.current_byte);
                    println!("Current chunk: {}", self.current_chunk);
                    let size = u64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + 8]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_byte += 8;
                    let value =
                        self.msg[self.current_byte..self.current_byte + size as usize].to_vec();
                    self.current_byte += size as usize;
                    self.current_chunk += 1;
                    value
                }
                Chunks::Any | Chunks::Rest => {
                    let value = self.msg[self.current_byte..self.current_byte + 8].to_vec();
                    self.current_byte += 8;
                    if let Chunks::Any = self.protocol[self.current_chunk] {
                        self.current_chunk += 1;
                    }
                    value
                }
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read a boolean from the message
        pub fn read_bool(&mut self) -> bool {
            if self.unchecked {
                let value = self.msg[self.current_byte] != 0;
                self.current_byte += 1;
                println!("Bool: {value}");
                return value;
            }
            match &self.protocol[self.current_chunk] {
                Chunks::Bool => {
                    let value = self.msg[self.current_byte] != 0;
                    self.current_byte += 1;
                    self.current_chunk += 1;
                    value
                }
                Chunks::Any | Chunks::Rest => {
                    let value = self.msg[self.current_byte] != 0;
                    self.current_byte += 8;
                    if let Chunks::Any = self.protocol[self.current_chunk] {
                        self.current_chunk += 1;
                    }
                    value
                }
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Finish reading the message
        pub fn finalize(&self) {
            if self.current_chunk != self.protocol.len() {
                panic!("Not all chunks were read");
            }
        }

        /// reset the reader
        pub fn reset(&mut self) {
            self.current_byte = 0;
            self.current_chunk = 0;
        }

        /// skips data in the current chunk
        pub fn skip(&mut self) {
            match &self.protocol[self.current_chunk] {
                Chunks::Int { size } => {
                    self.current_byte += *size as usize;
                    self.current_chunk += 1;
                }
                Chunks::Float => {
                    self.current_byte += 8;
                    self.current_chunk += 1;
                }
                Chunks::Uint { size } => {
                    self.current_byte += *size as usize;
                    self.current_chunk += 1;
                }
                Chunks::String => {
                    let size = u64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + 8]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_byte += 8 + size as usize;
                    self.current_chunk += 1;
                }
                Chunks::Binary => {
                    let size = u64::from_be_bytes(
                        self.msg[self.current_byte..self.current_byte + 8]
                            .try_into()
                            .unwrap(),
                    );
                    self.current_byte += 8 + size as usize;
                    self.current_chunk += 1;
                }
                Chunks::Bool => {
                    self.current_byte += 1;
                    self.current_chunk += 1;
                }
                Chunks::Any => {
                    self.current_byte += 8;
                    self.current_chunk += 1;
                }
                Chunks::Rest => {
                    self.current_byte += 8;
                }
            }
        }

        /// advances to the next chunk without skipping data
        pub fn advance(&mut self) {
            self.current_chunk += 1;
        }
    }

    /// Defines each chunk of the connection protocol that can be sent or received
    #[derive(Debug, PartialEq)]
    pub enum Chunks {
        /// Integer with a defined size
        Int { size: u8 },
        /// Float with a constant size of 8 bits
        Float,
        /// Unsigned integer with a defined size
        Uint { size: u8 },
        /// Unicode data Split into size section(8 bytes) and data section(size bytes)
        String,
        /// Binary data split into size section(8 bytes) and data section(size bytes)
        Binary,
        /// Boolean (1 byte)
        Bool,
        /// Any data (8 bytes)
        Any,
        /// Rest of the data (8 bytes per chunk)
        Rest,
    }

    /// Defines the status of a player
    #[derive(Debug, PartialEq, Clone)]
    pub enum PlayerStatus {
        Online,
        Offline,
        Away,
        Busy,
        InGame {
            game: GameKind,
            /// The time the game started
            time: u64,
        },
    }

    impl PlayerStatus {
        pub fn from_uint(value: u64) -> Self {
            match value {
                0 => PlayerStatus::Online,
                1 => PlayerStatus::Offline,
                2 => PlayerStatus::Away,
                3 => PlayerStatus::Busy,
                4 => PlayerStatus::InGame {
                    game: GameKind::Normal,
                    time: 0,
                },
                5 => PlayerStatus::InGame {
                    game: GameKind::Ranked,
                    time: 0,
                },
                _ => panic!("Invalid status"),
            }
        }

        pub fn to_uint(&self) -> u64 {
            match self {
                PlayerStatus::Online => 0,
                PlayerStatus::Offline => 1,
                PlayerStatus::Away => 2,
                PlayerStatus::Busy => 3,
                PlayerStatus::InGame { game, time: _ } => match game {
                    GameKind::Normal => 4,
                    GameKind::Ranked => 5,
                },
            }
        }
    }

    /// Defines the kind of game a player is in
    #[derive(Debug, PartialEq, Clone)]
    pub enum GameKind {
        Normal,
        Ranked,
    }

    pub struct FriendSummary {
        pub username: String,
        pub quote: String,
        pub status: PlayerStatus,
    }

    #[derive(Debug, PartialEq)]
    pub enum Message {
        Login { username: String, password: Vec<u8> },
        Register { username: String, password: Vec<u8> },
        ClientData(ClientData),

        Ok(u64, Option<Vec<u8>>),
        Error(u64, Option<Vec<u8>>),
    }

    impl Message {
        pub fn to_bytes(&self) -> Vec<u8> {
            fn combine(head: u64, body: Vec<u8>) -> Vec<u8> {
                let mut writer = ConnectionWriter::new(CONTAINER).unchecked();
                writer
                    .write_uint(head)
                    .write_binary(&body)
                    .finalize_unchecked()
            }
            match self {
                Message::Login { username, password } => {
                    let body = ConnectionWriter::new(LOGIN)
                        .write_string(username)
                        .write_binary(password)
                        .finalize();
                    combine(0, body)
                }
                Message::Register { username, password } => {
                    let body = ConnectionWriter::new(LOGIN)
                        .write_string(username)
                        .write_binary(password)
                        .finalize();
                    combine(1, body)
                }
                Message::Ok(id, data) => {
                    let mut writer = ConnectionWriter::new(STATUS);
                    writer.write_uint(*id);
                    match data {
                        Some(data) => {
                            writer.write_bool(true);
                            writer.write_binary(data);
                        }
                        None => {
                            writer.write_bool(false);
                        }
                    }
                    combine(2, writer.finalize_unchecked())
                }
                Message::Error(id, data) => {
                    let mut writer = ConnectionWriter::new(STATUS);
                    writer.write_uint(*id);
                    match data {
                        Some(data) => {
                            writer.write_bool(true);
                            writer.write_binary(data);
                        }
                        None => {
                            writer.write_bool(false);
                        }
                    }
                    combine(3, writer.finalize_unchecked())
                }
                Message::ClientData(data) => {
                    println!("good");
                    let bin = data.to_bytes();
                    println!("gooder");
                    combine(4, bin)
                }
            }
        }

        pub fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
            let mut reader = ConnectionReader::new(CONTAINER, bytes);
            let head = reader.read_uint();
            let body = reader.read_binary();
            match head {
                0 => {
                    println!("got login request");
                    println!("body: {:x?}", body);
                    let mut reader = ConnectionReader::new(LOGIN, &body);
                    let username = reader.read_string();
                    let password = reader.read_binary();
                    println!("username: {username}");
                    Ok(Message::Login { username, password })
                }
                1 => {
                    println!("got register request");
                    let mut reader = ConnectionReader::new(LOGIN, &body);
                    let username = reader.read_string();
                    let password = reader.read_binary();
                    Ok(Message::Register { username, password })
                }
                2 => {
                    println!("got ok response");
                    let mut reader = ConnectionReader::new(STATUS, &body);
                    let id = reader.read_uint();
                    let has_data = reader.read_bool();
                    let data = if has_data {
                        Some(reader.read_binary())
                    } else {
                        None
                    };
                    Ok(Message::Ok(id, data))
                }
                3 => {
                    println!("got error response");
                    let mut reader = ConnectionReader::new(STATUS, &body);
                    let id = reader.read_uint();
                    let has_data = reader.read_bool();
                    let data = if has_data {
                        Some(reader.read_binary())
                    } else {
                        None
                    };
                    Ok(Message::Error(id, data))
                }
                4 => {
                    println!("got client data");
                    match ClientData::from_bytes(&body) {
                        Ok(data) => Ok(Message::ClientData(data)),
                        Err(e) => Err(e),
                    }
                }
                _ => Err(MessageError::InvalidMessage),
            }
        }

        pub async fn read_stream(stream: &mut TcpStream) -> Result<Self, MessageError> {
            let mut data = Vec::new();
            let mut buffer = [0; 1024];
            loop {
                match stream.read(&mut buffer).await {
                    Ok(n) => {
                        data.extend_from_slice(&buffer[..n]);
                        if n < buffer.len() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from stream: {:?}", e);
                        return Err(MessageError::InvalidMessage);
                    }
                }
            }
            println!("Data: {:x?}", data);
            match Message::from_bytes(&data) {
                Ok(message) => Ok(message),
                Err(e) => {
                    eprintln!("Failed to parse response: {:?}", e);
                    Err(MessageError::InvalidMessage)
                }
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum MessageError {
        InvalidMessage,
        InvalidMessageBody,
    }

    #[derive(Debug, PartialEq)]
    pub struct Friend {
        pub username: String,
        pub status: PlayerStatus,
        pub quote: String,
    }

    #[derive(Debug, PartialEq)]
    pub struct ClientData {
        pub username: String,
        pub friends: Vec<Friend>,
        pub funds: u64,
        pub status: PlayerStatus,
        pub quote: String,
    }

    impl ClientData {
        pub fn to_bytes(&self) -> Vec<u8> {
            let mut writer = ConnectionWriter::new(CLIENT_DATA);
            writer.write_string(&self.username);
            let mut friends = Vec::new();
            for friend in &self.friends {
                let mut writer = ConnectionWriter::new(FRIEND_SUMMARY);
                let bin = writer
                    .write_string(&friend.username)
                    .write_string(&friend.quote)
                    .write_uint(friend.status.to_uint())
                    .finalize();
                friends.extend_from_slice(&bin);
            }
            writer.write_binary(&friends);
            writer.write_uint(self.funds);
            writer.write_uint(self.status.to_uint());
            writer.write_string(&self.quote);
            writer.finalize()
        }

        pub fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
            let mut reader = ConnectionReader::new(CLIENT_DATA, bytes);
            let username = reader.read_string();
            let friends_bin = reader.read_binary();
            let mut friends = Vec::new();
            let mut cur_friends = &friends_bin[..];
            while cur_friends.len() > 0 {
                let mut reader = ConnectionReader::new(FRIEND_SUMMARY, cur_friends);
                let username = reader.read_string();
                let quote = reader.read_string();
                let status = reader.read_uint();
                friends.push(Friend {
                    username,
                    status: PlayerStatus::from_uint(status),
                    quote,
                });
                cur_friends = &cur_friends[reader.current_byte..];
            }
            let funds = reader.read_uint();
            println!("good");
            let status = reader.read_uint();
            println!("goood");
            let quote = reader.read_string();
            println!("gooood");
            reader.finalize();
            Ok(ClientData {
                username,
                friends,
                funds,
                status: PlayerStatus::from_uint(status),
                quote,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::login::{PASSWORD_MAX, USERNAME_MAX};

    use super::*;

    #[test]
    fn test_login_validation() {
        const VALID_USERNAME: &str = "username";
        const VALID_PASSWORD: &str = "password";
        assert_eq!(
            login::validate(VALID_USERNAME, VALID_PASSWORD),
            login::LoginValidation::Valid
        );
        assert_eq!(
            login::validate("a", VALID_PASSWORD),
            login::LoginValidation::UsernameTooShort
        );
        assert_eq!(
            login::validate("a".repeat(USERNAME_MAX).as_str(), VALID_PASSWORD),
            login::LoginValidation::UsernameTooLong
        );
        assert_eq!(
            login::validate("username with space", VALID_PASSWORD),
            login::LoginValidation::UsernameContainsWhitespace
        );
        assert_eq!(
            login::validate(VALID_USERNAME, "a"),
            login::LoginValidation::PasswordTooShort
        );
        assert_eq!(
            login::validate(VALID_USERNAME, "a".repeat(PASSWORD_MAX).as_str()),
            login::LoginValidation::PasswordTooLong
        );
        assert_eq!(
            login::validate(VALID_USERNAME, "password with space"),
            login::LoginValidation::PasswordContainsWhitespace
        );
    }
}
