use crate::core::command::command::RPTCommand;
use crate::core::enums::command_types::commands::RPTCommandType;
use crate::telegram::callbacks::utils::utils::{get_client_by_id, RequestHandlerData};
use crate::telegram::ui::ui::show_pc_control_menu;
use std::sync::Arc;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageReplyMarkupSetters};
use teloxide::prelude::{ChatId, Requester, ResponseResult};

pub async fn handle_open_control_pc(data: &RequestHandlerData) -> ResponseResult<()>  {
    let (q, bot, state) = (data.q.clone(), data.bot.clone(), data.state.clone());

    let data = q.data.clone().unwrap();
    let id = data.trim_start_matches("pc_c|");
    let client = get_client_by_id(id.to_string(), Arc::clone(&state)).await;
    let replace = show_pc_control_menu(&client);
    let chat_id = ChatId(q.clone().from.id.0 as i64);
    let message = q.message.clone().unwrap().id();

    bot.edit_message_text(chat_id, message, replace.0).await?;
    bot.edit_message_reply_markup(chat_id, message).reply_markup(replace.1).await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn handle_tskmngr_off(data: &RequestHandlerData) -> ResponseResult<()>  {
    let (q, bot, state) = (data.q.clone(), data.bot.clone(), data.state.clone());

    let data = q.data.clone().unwrap();
    let id = data.trim_start_matches("off_tskmngr|");
    let client = get_client_by_id(id.to_string(), state).await;
    let command = RPTCommand {
        rpt_type: RPTCommandType::ChangeEnabledTskManager,
        data: vec![],
        flags: vec![],
    };
    client.channel.expect("failed get channel").send(command).await.expect("failed to send command");

    bot.answer_callback_query(q.id).text(t!("alert.default")).await?;
    Ok(())
}

pub async fn handle_shutdown_pc(data: &RequestHandlerData) -> ResponseResult<()>  {
    let (q, bot, state) = (data.q.clone(), data.bot.clone(), data.state.clone());

    let data = q.data.clone().unwrap();
    let id = data.trim_start_matches("shutdown_pc|");
    let client = get_client_by_id(id.to_string(), state).await;
    let command = RPTCommand {
        rpt_type: RPTCommandType::Shutdown,
        data: vec![],
        flags: vec![],
    };
    client.channel.expect("failed get channel").send(command).await.expect("failed to send command");

    bot.answer_callback_query(q.id).text(t!("alert.default")).await?;
    Ok(())
}

pub async fn handle_set_volume(data: &RequestHandlerData) -> ResponseResult<()>  {
    let (q, bot, state) = (data.q.clone(), data.bot.clone(), data.state.clone());

    let data = q.data.clone().unwrap();
    let a = data.trim_start_matches("set_volume|");
    let split = a.split("&&&");
    let vec = split.collect::<Vec<&str>>();
    let (id, volume) = (vec[0], vec[1]);
    let client = get_client_by_id(id.to_string(), state).await;
    let command = RPTCommand {
        rpt_type: RPTCommandType::ChangeAudioVolume,
        data: volume.parse::<u32>().expect("failed parse").to_be_bytes().to_vec(),
        flags: vec![],
    };
    client.channel.expect("failed get channel").send(command).await.expect("failed to send command");

    bot.answer_callback_query(q.id).text(t!("alert.default")).await?;
    Ok(())
}