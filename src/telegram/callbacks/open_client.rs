pub mod handler {
    use teloxide::prelude::{Requester, ResponseResult};
    use crate::telegram::callbacks::utils::utils::{open_client, RequestHandlerData};

    pub async fn f(data: &RequestHandlerData) -> ResponseResult<()>  {
        data.notify_dialog.reset().await.expect("Failed to reset notify dialog");
        data.change_cursor.reset().await.expect("Failed to reset change cursor");
        data.change_wallpaper.reset().await.expect("Failed to reset change wallpaper");

        let id = data.q.data.clone().unwrap().trim_start_matches("open_client|").parse().unwrap();
        open_client(&data.bot, id, data.state.clone(), &data.q).await;
        data.bot.answer_callback_query(data.q.id.clone()).await?;
        
        Ok(())
    }
}