pub mod change_wallpaper {
    use std::io::Cursor;
    use teloxide::Bot;
    use teloxide::dispatching::dialogue::InMemStorage;
    use teloxide::net::Download;
    use teloxide::prelude::{Dialogue, Message, Requester};
    use teloxide::requests::ResponseResult;
    use teloxide::types::{PhotoSize};
    use crate::app::app::{ RPTClient};

    async fn convert_photo_to_bytes(bot: &Bot, photos: &[PhotoSize]) -> Option<Vec<u8>> {
        let photo = photos.last()?;
        let file = bot.get_file(photo.file.id.clone()).await.ok()?;
        let mut buffer = Vec::new();
        bot.download_file(&file.path, &mut Cursor::new(&mut buffer)).await.ok()?;
        Some(buffer)
    }

    #[derive(Clone, Default)]
    pub struct ChangeWallpaper {
        pub data: Vec<u8>,
        pub client: RPTClient,
    }

    #[derive(Clone, Default)]
    pub enum ChangeWallpaperState {
        #[default]
        ChangeWallpaperIdle,

        ChangeWallpaperSend {change_wallpaper: ChangeWallpaper},
    }

    pub type ChangeWallpaperDialogue = Dialogue<ChangeWallpaperState, InMemStorage<ChangeWallpaperState>>;

    pub async fn wait_for_wallpaper(bot: Bot, dialogue: ChangeWallpaperDialogue, msg: Message, wallpaper_data: ChangeWallpaper) -> ResponseResult<()> {
        if let Some(wallpaper) = msg.photo() {
            let data = ChangeWallpaper {
                data: convert_photo_to_bytes(&bot, wallpaper).await.unwrap(),
                client: wallpaper_data.client,
            };
            dialogue.update(ChangeWallpaperState::ChangeWallpaperSend {change_wallpaper: data}).await.expect("failed updating wallpaper");
            bot.delete_message(msg.chat.id, msg.id).await?;
        };
        Ok(())
    }
}