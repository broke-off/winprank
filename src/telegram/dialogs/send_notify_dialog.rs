pub mod notify_dialog {
    use teloxide::Bot;
    use teloxide::dispatching::dialogue::InMemStorage;
    use teloxide::payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters};
    use teloxide::prelude::{Dialogue, Message, Requester, ResponseResult};
    use teloxide::types::{MessageId, ParseMode};
    use crate::telegram::ui::ui::show_notification_editor;

    #[derive(Default, Clone)]
    pub struct NotificationData {
        pub title: String,
        pub message: String,
        pub id: String,
    }

    impl NotificationData {
        pub fn set_id(&mut self, id: &str) -> NotificationData {
            self.id = id.to_string();
            self.clone()
        }
    }

    #[derive(Clone, Default)]
    pub enum NotificationEditorState {
        #[default]
        NotifyEditorIdle,

        NotifyEditorStart {notification_data: NotificationData, message_id: MessageId},
        NotifyEditorTitle {notification_data: NotificationData, message_id: MessageId},
        NotifyEditorText {notification_data: NotificationData, message_id: MessageId},
    }

    pub type NotificationEditorDialogue = Dialogue<NotificationEditorState, InMemStorage<NotificationEditorState>>;

    pub async fn notify_editor_title(bot: Bot, dialogue: NotificationEditorDialogue, msg: Message, (notification_data, message_id): (NotificationData, MessageId)) -> ResponseResult<()>{
        if let Some(new_title) = msg.text() {
            let new_data = NotificationData {
                title: new_title.to_string(),
                message: notification_data.message,
                id: notification_data.id,
            };
            bot.delete_message(msg.chat.id, msg.id).await?;

            let replace = show_notification_editor(Some(new_data.clone()));

            bot.edit_message_text(msg.chat.id, message_id, replace.0).parse_mode(ParseMode::Html).await?;
            bot.edit_message_reply_markup(msg.chat.id, message_id).reply_markup(replace.1.unwrap()).await?;
            dialogue.update(NotificationEditorState::NotifyEditorStart {notification_data: new_data, message_id}).await.expect("Dialogue update failed");
        }
        Ok(())
    }

    pub async fn notify_editor_body(bot: Bot, dialogue: NotificationEditorDialogue, msg: Message, (notification_data, message_id): (NotificationData, MessageId)) -> ResponseResult<()>{
        if let Some(new_body) = msg.text() {
            let new_data = NotificationData {
                title: notification_data.title,
                message: new_body.to_string(),
                id: notification_data.id,
            };
            bot.delete_message(msg.chat.id, msg.id).await?;

            let replace = show_notification_editor(Some(new_data.clone()));

            bot.edit_message_text(msg.chat.id, message_id, replace.0).parse_mode(ParseMode::Html).await?;
            bot.edit_message_reply_markup(msg.chat.id, message_id).reply_markup(replace.1.unwrap()).await?;
            dialogue.update(NotificationEditorState::NotifyEditorStart {notification_data: new_data, message_id}).await.expect("Dialogue update failed");
        }
        Ok(())
    }
}