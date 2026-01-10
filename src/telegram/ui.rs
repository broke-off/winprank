pub mod ui {
    use std::collections::HashMap;
    use std::sync::Arc;
    use country_emoji::flag;
    use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
    use tokio::sync::Mutex;
    use crate::app::app::{AppState, RPTClient, RPTScript};
    use crate::telegram::dialogs::send_notify_dialog::notify_dialog::NotificationData;

    pub fn show_notification_editor(client_id: String, data: Option<NotificationData>) -> (String, Option<InlineKeyboardMarkup>) {
        let data = data.clone().unwrap_or_default();
        let text = format!("📝 Отправка уведомления клиенту\n\n🔹 Заголовок: {}\n🔸 Текст: {}", data.title, data.message);
        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::callback("✏️ Заголовок", "notify_editor_edit_title"),
                InlineKeyboardButton::callback("✏️ Текст", "notify_editor_edit_text"),
            ],
            vec![InlineKeyboardButton::callback("🚀 Отправить клиенту", "send_notify")],
            vec![InlineKeyboardButton::callback("❌ Отменить отправку", format!("open_client|{}", data.id))],
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
        let text = format!("Доступные скрипты ({count})");

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
        keyboard.push(vec![InlineKeyboardButton::callback("🔙 Назад", format!("open_client|{}", client.id))]);

        (text, InlineKeyboardMarkup::new(keyboard))
    }

    pub fn show_send_change_wallpaper() -> (String, Option<InlineKeyboardMarkup>) {
        let text = String::from("🖼️ Отправьте фотографию для смены обоев!\n⚠️Если ваша фотография удалилась значит вы ее загрузили");
        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback("✅ Отправить", "send")],
            vec![InlineKeyboardButton::callback("❌ Отменить отправку", "cancel")]]);
        (text, Some(keyboard))
    }

    pub fn show_send_cursor() -> (String, InlineKeyboardMarkup) {
        let text = String::from("🖱️ Отправьте курсор для мыши в виде файла (.cur). \n ❌ Анимированные курсоры не поддерживаются\n⚠️Если ваш файл удалился значит вы его загрузили");
        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback("✅ Отправить", "send"), InlineKeyboardButton::callback("🔄 Сбросить курсор", "reset")],
            vec![InlineKeyboardButton::callback("🔻 Уменьшить размер до 1", "set_min"), InlineKeyboardButton::callback("🔺 Увеличить размер до 15", "set_max")],
            vec![InlineKeyboardButton::callback("❌ Отменить отправку", "exit")]
        ]);

        (text, keyboard)
    }

    pub async fn show_audio_ui(client_id: String, state: Arc<Mutex<AppState>>) -> (String, InlineKeyboardMarkup) {
        let text = String::from("🎵 Выберите звуковой файл");
        let mut keyboard = Vec::new();

        let audio_list: HashMap<i32, String> = {
            let state_lock = state.lock().await;
            state_lock.sounds.clone()
        };
        let buttons: Vec<InlineKeyboardButton> = audio_list.keys().map(|x| InlineKeyboardButton::callback(audio_list.get(x).expect("").split(".").collect::<Vec<&str>>()[0], format!("pl_au|{}&&&{}", client_id, x))).collect();
        for button in buttons.chunks(3) {
            keyboard.push(button.to_vec());
        }

        keyboard.push(vec![InlineKeyboardButton::callback("🔙 Назад", format!("open_client|{}", client_id))]);

        (text, InlineKeyboardMarkup::new(keyboard))
    }

    pub fn show_client_menu(client: &RPTClient) -> (String,InlineKeyboardMarkup) {
        let connected_fmt = client.connected.format("%Y-%m-%d %H:%M:%S").to_string();
        let id = &client.id;
        let keyboard: InlineKeyboardMarkup = InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback("📜 Запустить скрипт", format!("open_scripts|{id}")), InlineKeyboardButton::callback("🎧 Воспроизвести звук", format!("audio|{id}"))],
            vec![InlineKeyboardButton::callback("📨 Отправить сообщение", format!("send_message|{id}")),InlineKeyboardButton::callback("✏️ Настройки курсора", format!("change_cursor|{id}"))],
            vec![InlineKeyboardButton::callback("🖼️ Сменить обои", format!("change_wallpaper|{id}"))],
            vec![InlineKeyboardButton::callback("🔙 Вернуться к списку клиентов", "show_clients")]
        ]);

        let text = format!(
            "🧟 <b>Клиент</b>\n\n\
                     💻 <b>PC Name:</b> {pc}\n\
                     🌐 <b>IP:</b> {ip}\n\
                     🪟 <b>ОС:</b> Windows {win}\n\
                     🗺️ <b>Страна:</b> {country}\n\
                     📜 <b>Запущенные скрипты:</b> {runned_scripts}\n\
                     ⏳ <b>Подключился:</b> {time}",
            pc = client.pc_name,
            ip = client.address,
            win = client.win,
            time = connected_fmt,
            runned_scripts = client.runned_scripts.len(),
            country = format!("{} {}", flag(&*client.cc).unwrap(), client.country),
        );
        (text, keyboard)
    }
}