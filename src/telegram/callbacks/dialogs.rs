use crate::telegram::callbacks::utils::utils::{get_client_by_id, RequestHandlerData};
use crate::telegram::dialogs::change_cursor::change_cursor::{ChangeCursor, ChangeCursorState};
use crate::telegram::dialogs::change_wallpaper::change_wallpaper::{ChangeWallpaper, ChangeWallpaperState};
use crate::telegram::dialogs::send_notify_dialog::notify_dialog::{NotificationData, NotificationEditorState};
use crate::telegram::ui::ui::{show_notification_editor, show_send_change_wallpaper, show_send_cursor};
use std::sync::Arc;
use teloxide::payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters};
use teloxide::prelude::{ChatId, Requester, ResponseResult};
use teloxide::types::ParseMode;

pub async fn start_notification_dialog(data: &RequestHandlerData) -> ResponseResult<()> {
    let (q, bot, dialog, state) = (data.q.clone(), data.bot.clone(), data.notify_dialog.clone(), data.state.clone());

    let data = q.data.clone().unwrap();
    let id = data.trim_start_matches("send_message|");
    let notify_data = NotificationData::default().set_id(id);
    let client = get_client_by_id(id.to_string(), Arc::clone(&state)).await;
    let replace = show_notification_editor(Some(notify_data.clone()));
    let chat_id = ChatId(q.clone().from.id.0 as i64);
    let message = q.message.unwrap().id();

    dialog.update(NotificationEditorState::NotifyEditorStart {notification_data: notify_data, message_id: message}).await.expect("Failed");

    bot.edit_message_text(chat_id, message, replace.0).parse_mode(ParseMode::Html).await?;
    bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1.unwrap()).await?;
    bot.answer_callback_query(q.id).await?;
    Ok(())
}

pub async fn start_cursor_dialog(data: &RequestHandlerData) -> ResponseResult<()> {
    let (q, bot, dialog, state) = (data.q.clone(), data.bot.clone(), data.change_cursor.clone(), data.state.clone());

    let data = q.data.clone().unwrap();
    let id = data.trim_start_matches("change_cursor|");
    let client = get_client_by_id(id.to_string(), state).await;
    let replace = show_send_cursor();
    let chat_id = ChatId(q.clone().from.id.0 as i64);
    let message = q.message.clone().unwrap().id();
    let cursor = ChangeCursor {data: Vec::new(), client};

    dialog.update(ChangeCursorState::ChangeCursorSend {change_cursor: cursor}).await.expect("Failed");

    bot.edit_message_text(chat_id, message, replace.0).await.expect("");
    bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await.expect("");
    bot.answer_callback_query(q.id).await?;
    Ok(())
}

pub async fn start_wallpaper_dialog(data: &RequestHandlerData) -> ResponseResult<()> {
    let (q, bot, dialog, state) = (data.q.clone(), data.bot.clone(), data.change_wallpaper.clone(), data.state.clone());

    let data = q.data.clone().unwrap();
    let id = data.trim_start_matches("change_wallpaper|");
    let client = get_client_by_id(id.to_string(), Arc::clone(&state)).await;
    let replace = show_send_change_wallpaper();
    let chat_id = ChatId(q.clone().from.id.0 as i64);
    let message = q.message.unwrap().id();
    let a = ChangeWallpaper { data: vec![], client };

    dialog.update(ChangeWallpaperState::ChangeWallpaperSend {change_wallpaper: a}).await.expect("Failed");

    bot.edit_message_text(chat_id, message, replace.0).await?;
    bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1.unwrap()).await?;
    bot.answer_callback_query(q.id).await?;
    Ok(())
}