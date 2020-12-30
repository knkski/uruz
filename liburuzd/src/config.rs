pub struct Config {
    pub database_path: String,
    pub api_host: [u8; 4],
    pub api_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: "uruz.sled".into(),
            api_host: [0, 0, 0, 0],
            api_port: 8000,
        }
    }
}
