pub mod change_cursor {
    use std::io::Cursor;
    use teloxide::Bot;
    use teloxide::dispatching::dialogue::InMemStorage;
    use teloxide::net::Download;
    use teloxide::prelude::{Dialogue, Message, Requester, ResponseResult};
    use crate::app::app::RPTClient;
    use crate::telegram::dialogs::change_cursor::change_cursor::ChangeCursorState::ChangeCursorSend;

    #[derive(Clone, Default)]
    pub struct ChangeCursor {
        pub data: Vec<u8>,
        pub client: RPTClient
    }

    #[derive(Clone, Default)]
    pub enum ChangeCursorState {
        #[default]
        ChangeCursorIdle,

        ChangeCursorSend {change_cursor: ChangeCursor },
    }

    async fn download_document_to_vec(bot: &Bot, msg: &Message) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let doc = msg.document().unwrap();
        let file = bot.get_file(doc.file.id.clone()).await?;
        let mut buffer = Vec::new();
        bot.download_file(&file.path, &mut Cursor::new(&mut buffer)).await?;

        Ok(buffer)
    }

    pub type ChangeCursorDialogue = Dialogue<ChangeCursorState, InMemStorage<ChangeCursorState>>;

    pub async fn wait_for_cursor(bot: Bot, dialogue: ChangeCursorDialogue, msg: Message, data: ChangeCursor) -> ResponseResult<()> {
        if let Some(cursor) = msg.document() {
            let data = ChangeCursor {
                data: download_document_to_vec(&bot, &msg).await.unwrap(),
                client: data.client,
            };
            dialogue.update(ChangeCursorSend {change_cursor: data}).await.expect("failed updating wallpaper");
            bot.delete_message(msg.chat.id, msg.id).await?;
        }

        Ok(())
    }
}