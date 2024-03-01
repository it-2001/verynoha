

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
                LoginValidation::UsernameContainsWhitespace => write!(f, "Username contains whitespace"),
                LoginValidation::PasswordTooShort => write!(f, "Password is too short"),
                LoginValidation::PasswordTooLong => write!(f, "Password is too long"),
                LoginValidation::PasswordContainsWhitespace => write!(f, "Password contains whitespace"),
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
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(password);
        hasher.finalize().to_vec()
    }
}


/// helper module for validating and defining the connection protocol
pub mod connection_protocol {
    pub const LOGIN : &'static [Chunks] = &[
        // username
        Chunks::String,
        // password hash
        Chunks::Binary,
    ];
    /// Holds the connection protocol
    pub struct ConnectionProtocol {
        pub chunks: &'static [Chunks],
    }

    /// Holds the data neccesary to write a message to the connection
    pub struct ConnectionWriter {
        pub current_chunk: usize,
        pub msg: Vec<u8>,
        pub protocol: &'static [Chunks],
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
            }
        }

        /// Write an integer to the message
        pub fn write_int(&mut self, value: i64) -> &mut Self {
            match &self.protocol[self.current_chunk] {
                Chunks::Int{size} => {
                    self.msg.extend_from_slice(&value.to_be_bytes()[8 - *size as usize..]);
                    self.current_chunk += 1;
                },
                _ => panic!("Invalid chunk type, expected {:?}, got int", self.protocol[self.current_chunk]),
            }
            self
        }

        /// Write a float to the message
        pub fn write_float(&mut self, value: f64) -> &mut Self {
            match &self.protocol[self.current_chunk] {
                Chunks::Float => {
                    self.msg.extend_from_slice(&value.to_be_bytes());
                    self.current_chunk += 1;
                },
                _ => panic!("Invalid chunk type, expected {:?}, got float", self.protocol[self.current_chunk]),
            }
            self
        }

        /// Write an unsigned integer to the message
        pub fn write_uint(&mut self, value: u64) -> &mut Self {
            match &self.protocol[self.current_chunk] {
                Chunks::Uint{size} => {
                    self.msg.extend_from_slice(&value.to_be_bytes()[8 - *size as usize..]);
                    self.current_chunk += 1;
                },
                _ => panic!("Invalid chunk type, expected {:?}, got uint", self.protocol[self.current_chunk]),
            }
            self
        }

        /// Write a string to the message
        pub fn write_string(&mut self, value: &str) -> &mut Self {
            match &self.protocol[self.current_chunk] {
                Chunks::String => {
                    let value = value.as_bytes();
                    let size = value.len() as u64;
                    self.msg.extend_from_slice(&size.to_be_bytes());
                    self.msg.extend_from_slice(value);
                    self.current_chunk += 1;
                },
                _ => panic!("Invalid chunk type, expected {:?}, got string", self.protocol[self.current_chunk]),
            }
            self
        }

        /// Write a binary to the message
        pub fn write_binary(&mut self, value: &[u8]) -> &mut Self {
            match &self.protocol[self.current_chunk] {
                Chunks::Binary => {
                    let size = value.len() as u64;
                    self.msg.extend_from_slice(&size.to_be_bytes());
                    self.msg.extend_from_slice(value);
                    self.current_chunk += 1;
                },
                _ => panic!("Invalid chunk type, expected {:?}, got binary", self.protocol[self.current_chunk]),
            }
            self
        }

        /// Write a boolean to the message
        pub fn write_bool(&mut self, value: bool) -> &mut Self {
            match &self.protocol[self.current_chunk] {
                Chunks::Bool => {
                    self.msg.push(if value {1} else {0});
                    self.current_chunk += 1;
                },
                _ => panic!("Invalid chunk type, expected {:?}, got bool", self.protocol[self.current_chunk]),
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
    }


    /// Holds the data neccesary to read a message from the connection
    pub struct ConnectionReader <'a> {
        pub current_byte: usize,
        pub current_chunk: usize,
        pub msg: &'a [u8],
        pub protocol: &'static [Chunks],
    }

    impl <'a> ConnectionReader <'a> {
        /// Create a new connection reader
        pub fn new(protocol: &'static [Chunks], msg: &'a [u8]) -> ConnectionReader<'a> {
            ConnectionReader {
                current_byte: 0,
                current_chunk: 0,
                msg,
                protocol,
            }
        }

        /// Read an integer from the message
        pub fn read_int(&mut self) -> i64 {
            match &self.protocol[self.current_chunk] {
                Chunks::Int{size} => {
                    let value = i64::from_be_bytes(self.msg[self.current_byte..self.current_byte + *size as usize].try_into().unwrap());
                    self.current_chunk += 1;
                    self.current_byte += *size as usize;
                    value
                },
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read a float from the message
        pub fn read_float(&mut self) -> f64 {
            match &self.protocol[self.current_chunk] {
                Chunks::Float => {
                    let value = f64::from_be_bytes(self.msg[self.current_byte..self.current_byte + 8].try_into().unwrap());
                    self.current_byte += 8;
                    self.current_chunk += 1;
                    value
                },
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read an unsigned integer from the message
        pub fn read_uint(&mut self) -> u64 {
            match &self.protocol[self.current_chunk] {
                Chunks::Uint{size} => {
                    let value = u64::from_be_bytes(self.msg[self.current_byte..self.current_byte + *size as usize].try_into().unwrap());
                    self.current_byte += *size as usize;
                    self.current_chunk += 1;
                    value
                },
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read a string from the message
        pub fn read_string(&mut self) -> String {
            match &self.protocol[self.current_chunk] {
                Chunks::String => {
                    let size = u64::from_be_bytes(self.msg[self.current_byte..self.current_byte + 8].try_into().unwrap());
                    self.current_byte += 8;
                    let value = String::from_utf8(self.msg[self.current_byte..self.current_byte + size as usize].to_vec()).unwrap();
                    self.current_byte += size as usize;
                    self.current_chunk += 1;
                    value
                },
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read a binary from the message
        pub fn read_binary(&mut self) -> Vec<u8> {
            match &self.protocol[self.current_chunk] {
                Chunks::Binary => {
                    let size = u64::from_be_bytes(self.msg[self.current_byte..self.current_byte + 8].try_into().unwrap());
                    self.current_byte += 8;
                    let value = self.msg[self.current_byte..self.current_byte + size as usize].to_vec();
                    self.current_byte += size as usize;
                    self.current_chunk += 1;
                    value
                },
                _ => panic!("Invalid chunk type"),
            }
        }

        /// Read a boolean from the message
        pub fn read_bool(&mut self) -> bool {
            match &self.protocol[self.current_chunk] {
                Chunks::Bool => {
                    let value = self.msg[self.current_byte] != 0;
                    self.current_byte += 1;
                    self.current_chunk += 1;
                    value
                },
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
                Chunks::Int{size} => {
                    self.current_byte += *size as usize;
                    self.current_chunk += 1;
                },
                Chunks::Float => {
                    self.current_byte += 8;
                    self.current_chunk += 1;
                },
                Chunks::Uint{size} => {
                    self.current_byte += *size as usize;
                    self.current_chunk += 1;
                },
                Chunks::String => {
                    let size = u64::from_be_bytes(self.msg[self.current_byte..self.current_byte + 8].try_into().unwrap());
                    self.current_byte += 8 + size as usize;
                    self.current_chunk += 1;
                },
                Chunks::Binary => {
                    let size = u64::from_be_bytes(self.msg[self.current_byte..self.current_byte + 8].try_into().unwrap());
                    self.current_byte += 8 + size as usize;
                    self.current_chunk += 1;
                },
                Chunks::Bool => {
                    self.current_byte += 1;
                    self.current_chunk += 1;
                },
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
        Int{
            size: u8,
        },
        /// Float with a constant size of 8 bits
        Float,
        /// Unsigned integer with a defined size
        Uint{
            size: u8,
        },
        /// Unicode data Split into size section(8 bytes) and data section(size bytes)
        String,
        /// Binary data split into size section(8 bytes) and data section(size bytes)
        Binary,
        /// Boolean (1 byte)
        Bool,
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
        assert_eq!(login::validate(VALID_USERNAME, VALID_PASSWORD), login::LoginValidation::Valid);
        assert_eq!(login::validate("a", VALID_PASSWORD), login::LoginValidation::UsernameTooShort);
        assert_eq!(login::validate("a".repeat(USERNAME_MAX).as_str(), VALID_PASSWORD), login::LoginValidation::UsernameTooLong);
        assert_eq!(login::validate("username with space", VALID_PASSWORD), login::LoginValidation::UsernameContainsWhitespace);
        assert_eq!(login::validate(VALID_USERNAME, "a"), login::LoginValidation::PasswordTooShort);
        assert_eq!(login::validate(VALID_USERNAME,"a".repeat(PASSWORD_MAX).as_str()), login::LoginValidation::PasswordTooLong);
        assert_eq!(login::validate(VALID_USERNAME, "password with space"), login::LoginValidation::PasswordContainsWhitespace);

    }
}
