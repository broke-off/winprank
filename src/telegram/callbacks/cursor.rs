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
use crate::telegram::dialogs::change_cursor::change_cursor::{ChangeCursor, ChangeCursorDialogue};

pub async fn change_cursor_callbacks(
    bot: Bot, q: CallbackQuery, state: Arc<Mutex<AppState>>, dialogue: ChangeCursorDialogue, change_cursor_data: ChangeCursor
) -> ResponseResult<()> {
    if let Some(data) = q.clone().data {
        let client = change_cursor_data.client;
        if data == "send" && !change_cursor_data.data.is_empty() {
            let command = RPTCommand { rpt_type: RPTCommandType::ChangeCursor, data: change_cursor_data.data, flags: vec![] };
            client.channel.unwrap().send(command).await.unwrap();
            open_client(&bot, client.id, state, &q).await;
            dialogue.exit().await.expect("Failed to exit");
            bot.answer_callback_query(q.id.clone()).show_alert(true).text(t!("alert.cursor_was_modified")).await?;
        }
        else if data == "exit" {
            dialogue.exit().await.expect("Failed to exit");
            open_client(&bot, client.id, state, &q).await;
            bot.answer_callback_query(q.id).await?;
        }
        else if data == "set_max" || data == "set_min" {
            let size = if data == "set_max" { 15_u32 } else { 1_u32 };
            let command = RPTCommand {
                rpt_type: RPTCommandType::ChangeCursor,
                data: Vec::from(size.to_be_bytes()),
                flags: vec!["SET_SIZE".to_string()],
            };
            client.channel.unwrap().send(command).await.unwrap();
            open_client(&bot, client.id, state, &q).await;
            dialogue.exit().await.expect("Failed to exit");
            bot.answer_callback_query(q.id.clone()).show_alert(true).text(t!("alert.cursor_was_modified")).await?;
        }
        else if data == "reset" {
            let command = RPTCommand { rpt_type: RPTCommandType::ChangeCursor, data: Vec::new(), flags: vec![String::from("RESET")] };
            client.channel.expect("failed get channel").send(command).await.unwrap();
            dialogue.exit().await.expect("Failed to exit");
            open_client(&bot, client.id, state, &q).await;
            bot.answer_callback_query(q.id.clone()).show_alert(true).text(t!("alert.cursor_was_reset")).await?;
        }
    }
    Ok(())
}