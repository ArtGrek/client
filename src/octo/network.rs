use serde_json::{Value, Map};
use tokio;
use tokio::time::Instant;
use super::Game;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;

pub async fn send_exec(cmd_type: &str, a_game: &mut Game) -> Result<(), Box<dyn std::error::Error>>  {
    let mut fields: Map<String, Value> = Default::default();
    fields.insert("type".to_string(), Value::String(cmd_type.to_string()));
    match cmd_type {
        "shutdown" => {}
        "get_last_state" => {}
        "api" => {
            let mut url = a_game.request.url.clone();
            if !a_game.request.params.is_empty() {
                let params: Vec<String> = a_game.request.params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                if url.contains('?') {url.push('&');} else {url.push('?');}
                url.push_str(&params.join("&"));
            }
            let mut data = Map::new();
            data.insert("url".to_string(), Value::String(url));
            data.insert("body".to_string(), a_game.request.body.clone());
            data.insert("headers".to_string(), Value::Object(Map::new()));
            fields.insert("data".to_string(), Value::Object(data));
        }
        _ => {}
    }
    let msg = Value::Object(fields.clone());
    let max_retries = 10; 
    let mut attempts = 0;
    while attempts < max_retries {
        let start_time = Instant::now();
        match connect_async(a_game.ws_client.as_str()).await {
            Ok((mut ws_stream, _)) => {
                let send_res = ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(msg.to_string().into())).await;
                if let Err(err) = send_res {
                    print!("\x1B[1A\x1B[2K");
                    print!("\x1B[1A\x1B[2K");
                    eprintln!("\r\t[WARN] Send failed: {}. Attempt {}/{}", err, attempts + 1, max_retries);
                    print!("\x1B[K\t\n");
                    print!("\x1B[K\t\n");
                    attempts += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    continue;
                }

                if let Some(msg) = ws_stream.next().await {
                    match msg {
                        Ok(tokio_tungstenite::tungstenite::Message::Text(txt)) => {
                            //a_game.data.prev_request_id = a_game.data.request_id.clone();
                            a_game.data.prev_client_command_time = Some(start_time.elapsed().as_millis() as i64);
                            a_game.response = serde_json::from_str(&txt).unwrap_or_else(|_| Value::Null);
                            return Ok(());
                        }
                        Ok(_) => {
                            print!("\x1B[1A\x1B[2K");
                            print!("\x1B[1A\x1B[2K");
                            eprintln!("\r\t[WARN] Unexpected message type. Attempt {}/{}", attempts + 1, max_retries);
                            print!("\x1B[K\t\n");
                            print!("\x1B[K\t\n");
                        }
                        Err(err) => {
                            print!("\x1B[1A\x1B[2K");
                            print!("\x1B[1A\x1B[2K");
                            eprintln!("\r\t[WARN] Receive failed: {}. Attempt {}/{}", err, attempts + 1, max_retries);
                            print!("\x1B[K\t\n");
                            print!("\x1B[K\t\n");
                        }
                    }
                } else {
                    print!("\x1B[1A\x1B[2K");
                    print!("\x1B[1A\x1B[2K");
                    eprintln!("\r\t[WARN] No response from server. Attempt {}/{}", attempts + 1, max_retries);
                    print!("\x1B[K\t\n");
                    print!("\x1B[K\t\n");
                }
            }
            Err(err) => {
                print!("\x1B[1A\x1B[2K");
                print!("\x1B[1A\x1B[2K");
                eprintln!("\r\t[WARN] Connection failed: {}. Attempt {}/{}", err, attempts + 1, max_retries);
                print!("\x1B[K\t\n");
                print!("\x1B[K\t\n");
            }
        }
        attempts += 1;
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
    print!("\x1B[1A\x1B[2K");
    print!("\x1B[1A\x1B[2K");
    eprintln!("\r\t[ERROR] Maximum number of attempts ({}) exceeded!", max_retries);
    Err("Maximum number of attempts exceeded!".into())
}



        //Connection error: IO error: Удаленный хост принудительно разорвал существующее подключение. (os error 10054). attempts 1/10
        //Connection error: IO error: Подключение не установлено, т.к. конечный компьютер отверг запрос на подключение. (os error 10061). attempts 2/10