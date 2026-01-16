use crate::app::app::{RPTClient, SCRIPTS_PATH};
use crate::core::command::command::RPTCommand;
use crate::core::enums::command_types::commands::RPTCommandType;
use crate::telegram::callbacks::utils::utils::{get_client_by_id, RequestHandlerData};
use crate::telegram::ui::ui::show_scripts;

use rust_i18n::t;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageReplyMarkupSetters};
use teloxide::prelude::{ChatId, Requester, ResponseResult};

pub async fn handle_open_scripts(data: &RequestHandlerData) -> ResponseResult<()> {
    let (q, bot, state) = (data.q.clone(), data.bot.clone(), data.state.clone());

    let data = q.data.clone().unwrap();
    let id = data.trim_start_matches("open_scripts|");
    let client = get_client_by_id(id.to_string(), Arc::clone(&state)).await;
    let replace = show_scripts(client, Arc::clone(&state)).await;
    let chat_id = ChatId(q.clone().from.id.0 as i64);
    let message = q.message.clone().unwrap().id();

    bot.edit_message_text(chat_id, message, replace.0).await?;
    bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn handle_run_script(data: &RequestHandlerData) -> ResponseResult<()> {
    let (q, bot, state) = (data.q.clone(), data.bot.clone(), data.state.clone());

    let data = q.data.clone().unwrap();
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

    let chat_id = ChatId(q.clone().from.id.0 as i64);
    let message = q.message.unwrap().id();
    let replace = show_scripts(client.clone(), Arc::clone(&state)).await;
    bot.edit_message_text(chat_id, message, replace.0).await?;
    bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await?;

    client.channel.unwrap().send(command).await.expect("Failed to send command");

    {
        let mut state = state.lock().await;
        let client_1: &mut RPTClient = state.clients.iter_mut().find(|x| x.id == client_id).unwrap();
        client_1.runned_scripts.push(script_id.to_string());
    }

    bot.answer_callback_query(q.id).show_alert(true).text(t!("alert.scripts_was_launched")).await?;
    Ok(())
}