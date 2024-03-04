use std::net::IpAddr;




pub struct Options {
    pub username: String,
    pub password: Vec<u8>,
    pub auto_login: bool,
    pub server_ip: IpAddr,
}

impl Options {
    pub fn default() -> Options {
        Options {
            username: String::from(""),
            password: Vec::new(),
            auto_login: false,
            server_ip: common::DEFAULT_SERVER_IP.parse().unwrap(),
        }
    }
    
    pub fn load() -> Option<Options> {
        let file = match std::fs::read_to_string("config.toml") {
            Ok(file) => file,
            Err(_) =>  return None,
        };
        let mut reader = common::connection_protocol::ConnectionReader::new(&common::connection_protocol::LOAD, &file.as_bytes());
        let options= Options {
            username: reader.read_string(),
            password: reader.read_binary(),
            auto_login: reader.read_bool(),
            server_ip: match reader.read_string().parse() {
                Ok(ip) => ip,
                Err(_) => common::DEFAULT_SERVER_IP.parse().unwrap(),
            },
        };
        Some(options)
    }

    pub fn save(&self) {
        let mut writer = common::connection_protocol::ConnectionWriter::new(common::connection_protocol::LOAD);
        writer.write_string(&self.username)
            .write_binary(&self.password)
            .write_bool(self.auto_login)
            .write_string(&self.server_ip.to_string());
        std::fs::write("config.toml", writer.finalize()).unwrap();
    }
}