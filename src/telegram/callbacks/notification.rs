use std::sync::Arc;
use teloxide::Bot;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::prelude::{CallbackQuery, Requester, ResponseResult};
use teloxide::types::MessageId;
use tokio::sync::Mutex;
use rust_i18n::t;
use crate::app::app::AppState;
use crate::core::command::command::RPTCommand;
use crate::core::enums::command_types::commands::RPTCommandType;
use crate::telegram::callbacks::utils::utils::{get_client_by_id, open_client};
use crate::telegram::dialogs::send_notify_dialog::notify_dialog::{NotificationData, NotificationEditorDialogue, NotificationEditorState};
pub async fn notify_editor_callbacks(bot: Bot, q: CallbackQuery, state: Arc<Mutex<AppState>>, dialogue: NotificationEditorDialogue, (notification_data, _): (NotificationData, MessageId)) -> ResponseResult<()> {
    if let Some(data) = q.clone().data {
        if data == "notify_editor_edit_title" {
            let message = q.message.unwrap().id();
            dialogue.update(NotificationEditorState::NotifyEditorTitle {notification_data, message_id: message}).await.expect("Failed");
            bot.answer_callback_query(q.id).await?;
        }
        else if data == "notify_editor_edit_text" {
            let message = q.message.unwrap().id();
            dialogue.update(NotificationEditorState::NotifyEditorText {notification_data, message_id: message}).await.expect("Failed");
            bot.answer_callback_query(q.id).await?;
        }
        else if data.starts_with("open_client|") {
            dialogue.reset().await.expect("Failed to reset");
            let id = data.trim_start_matches("open_client|");
            open_client(&bot, id.parse().unwrap(), state, &q).await;
            bot.answer_callback_query(q.id).await?;
        }
        else if data.starts_with("send_notify") {
            let (id, title, data) = (notification_data.id, notification_data.title, notification_data.message);
            let client = get_client_by_id(id.to_string(), Arc::clone(&state)).await;
            let command = RPTCommand {
                rpt_type: RPTCommandType::WinMessage,
                data: format!("{}<split>{}", title, data).into_bytes(),
                flags: vec![],
            };
            client.channel.unwrap().send(command).await.unwrap();
            bot.answer_callback_query(q.id).show_alert(true).text(t!("alert.message_send")).await?;
        }
    }
    Ok(())
}