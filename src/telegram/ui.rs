pub mod ui {
    use std::collections::HashMap;
    use std::sync::Arc;
    use country_emoji::flag;
    use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
    use tokio::sync::Mutex;
    use crate::app::app::{AppState, RPTClient};
    use crate::telegram::dialogs::send_notify_dialog::notify_dialog::NotificationData;
    use rust_i18n::t;

    pub fn show_notification_editor(client_id: String, data: Option<NotificationData>) -> (String, Option<InlineKeyboardMarkup>) {
        let data = data.clone().unwrap_or_default();
        let text = t!("menu.send_notify", "title" => data.title, "text" => data.message).to_string();
        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::callback(t!("button.change_title"), "notify_editor_edit_title"),
                InlineKeyboardButton::callback(t!("button.change_text"), "notify_editor_edit_text"),
            ],
            vec![InlineKeyboardButton::callback(t!("button.send"), "send_notify")],
            vec![InlineKeyboardButton::callback(t!("button.cancel_sending"), format!("open_client|{}", data.id))],
        ]);

        (text, Some(keyboard))
    }

    pub fn is_runned(client: RPTClient, name: String) -> String {
        if client.runned_scripts.contains(&name) {
            return String::from("❌")
        }
        String::new()
    }

    pub async fn show_scripts(client: RPTClient, state: Arc<Mutex<AppState>>) -> (String, InlineKeyboardMarkup) {
        let state_lock = state.lock().await;
        let scripts = state_lock.scripts.as_ref();
        let count = scripts.map(|s| s.len()).unwrap_or(0);
        let text = t!("menu.scripts", "count" => count).to_string();

        let mut keyboard = Vec::new();

        if let Some(scripts_map) = scripts {
            let buttons: Vec<InlineKeyboardButton> = scripts_map
                .keys()
                .map(|name| InlineKeyboardButton::callback(format!("{}{}", is_runned(client.clone(), name.clone()), scripts.unwrap().get(name).unwrap().name), format!("run_script|{name}&&&{}", client.id)))
                .collect();

            for chunk in buttons.chunks(2) {
                keyboard.push(chunk.to_vec());
            }
        }
        keyboard.push(vec![InlineKeyboardButton::callback(t!("button.back"), format!("open_client|{}", client.id))]);

        (text, InlineKeyboardMarkup::new(keyboard))
    }

    pub fn show_send_change_wallpaper() -> (String, Option<InlineKeyboardMarkup>) {
        let text = t!("menu.change_wallpaper").to_string();
        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback(t!("button.send"), "send")],
            vec![InlineKeyboardButton::callback(t!("button.cancel_sending"), "cancel")]]);
        (text, Some(keyboard))
    }

    pub fn show_send_cursor() -> (String, InlineKeyboardMarkup) {
        let text = t!("menu.change_cursor").to_string();
        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback(t!("button.send"), "send"), InlineKeyboardButton::callback(t!("button.reset_cursor"), "reset")],
            vec![InlineKeyboardButton::callback(t!("button.change_cursor_to_min"), "set_min"), InlineKeyboardButton::callback(t!("button.change_cursor_to_max"), "set_max")],
            vec![InlineKeyboardButton::callback(t!("button.cancel_sending"), "exit")]
        ]);

        (text, keyboard)
    }

    pub async fn show_audio_ui(client_id: String, state: Arc<Mutex<AppState>>) -> (String, InlineKeyboardMarkup) {
        let text = t!("menu.play_audio").to_string();
        let mut keyboard = Vec::new();

        let audio_list: HashMap<i32, String> = {
            let state_lock = state.lock().await;
            state_lock.sounds.clone()
        };
        let buttons: Vec<InlineKeyboardButton> = audio_list.keys().map(|x| InlineKeyboardButton::callback(audio_list.get(x).expect("").split(".").collect::<Vec<&str>>()[0], format!("pl_au|{}&&&{}", client_id, x))).collect();
        for button in buttons.chunks(3) {
            keyboard.push(button.to_vec());
        }

        keyboard.push(vec![InlineKeyboardButton::callback(t!("button.back"), format!("open_client|{}", client_id))]);

        (text, InlineKeyboardMarkup::new(keyboard))
    }

    pub fn show_client_menu(client: &RPTClient) -> (String,InlineKeyboardMarkup) {
        let connected_fmt = client.connected.format("%Y-%m-%d %H:%M:%S").to_string();
        let id = &client.id;
        let keyboard: InlineKeyboardMarkup = InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback(t!("button.run_script"), format!("open_scripts|{id}")), InlineKeyboardButton::callback(t!("button.play_audio"), format!("audio|{id}"))],
            vec![InlineKeyboardButton::callback(t!("button.send_notify"), format!("send_message|{id}")),InlineKeyboardButton::callback(t!("button.change_cursor"), format!("change_cursor|{id}"))],
            vec![InlineKeyboardButton::callback(t!("button.change_wallpaper"), format!("change_wallpaper|{id}"))],
            vec![InlineKeyboardButton::callback(t!("button.back_to_clients"), "show_clients")]
        ]);

        let text = t!("client.info",
            "pc" => client.pc_name,
            "ip" => client.address,
            "win" => client.win,
            "time" => connected_fmt,
            "runned_scripts" => client.runned_scripts.len(),
            "country" => format!("{} {}", flag(&*client.cc).unwrap(), client.country),
        );
        (text.parse().unwrap(), keyboard)
    }
}