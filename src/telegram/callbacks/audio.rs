use crate::core::command::command::RPTCommand;
use crate::core::enums::command_types::commands::RPTCommandType;
use crate::telegram::callbacks::utils::utils::{get_client_by_id, RequestHandlerData};
use crate::telegram::ui::ui::show_audio_ui;
use rust_i18n::t;
use std::fs;
use std::sync::Arc;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageReplyMarkupSetters};
use teloxide::prelude::{ChatId, Requester, ResponseResult};

pub async fn handle_open_audio(data: &RequestHandlerData) -> ResponseResult<()> {
    let (q, bot, state) = (data.q.clone(), data.bot.clone(), data.state.clone());
    
    let data = q.data.clone().unwrap();
    let id = data.trim_start_matches("audio|");
    let chat_id = ChatId(q.clone().from.id.0 as i64);
    let message = q.message.unwrap().id();
    let replace = show_audio_ui(id.to_string(), Arc::clone(&state)).await;

    bot.edit_message_text(chat_id, message, replace.0).await?;
    bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await?;
    bot.answer_callback_query(q.id).await?;
    Ok(())
}

pub async fn handle_play_audio(data: &RequestHandlerData) -> ResponseResult<()> {
    let (q, bot, state) = (data.q.clone(), data.bot.clone(), data.state.clone());
    
    let data = q.data.clone().unwrap();
    let send_data = data.trim_start_matches("pl_au|");
    let new_send_data = send_data.split("&&&").collect::<Vec<&str>>();
    let (client_id, file_id) = (new_send_data[0], new_send_data[1]);
    let sound_list = {
        let app = state.lock().await;
        app.sounds.clone()
    };

    let mut client = get_client_by_id(client_id.to_string(), Arc::clone(&state)).await;
    let audio_file = fs::read(format!("sounds/{}", sound_list.get(&file_id.parse::<i32>().unwrap()).expect("failed"))).expect("");

    let command = RPTCommand {
        rpt_type: RPTCommandType::PlayAudio,
        data: audio_file,
        flags: vec![],
    };
    client.channel.unwrap().send(command).await.expect("Failed to send command");
    bot.answer_callback_query(q.id).show_alert(true).text(t!("alert.sound_was_played")).await?;
    Ok(())
}