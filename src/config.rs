pub mod config {
    use serde::Deserialize;

    #[derive(Deserialize, Clone)]
    pub struct Config {
        pub telegram: TelegramBotPanel,
        pub server: Server
    }

    #[derive(Deserialize, Clone)]
    pub struct Server {
       pub ip: String,
       pub port: String,
       pub ping_interval: u32,
       pub data_secure_key: String 
    }

    #[derive(Deserialize, Clone)]
    pub struct TelegramBotPanel {
        pub token: String,
        pub admin_chat_id: i64,
        pub lang: String,
    }
}