use reqwest::{Client, redirect::Policy};
use serde_json::{json, Value};
use std::path::Path;
use std::fs;
use tokio;
use tokio::time::Duration;
use std::collections::HashMap;
use tokio::time::Instant;
use super::{Game, GameData};
use anyhow::Result;
use url::Url;
use uuid::Uuid;

pub async fn launch_game_v1(a_launch_url: String, a_location: String, a_game_name: String, a_device: String, default_data: &mut GameData) -> Result<String> {
    match reqwest::get(a_launch_url).await {
        Ok(response) => match response.text().await {
            Ok(game_html) => {
                let game_html_location = a_location.to_owned() + &a_game_name.clone() + "/index.html";
                if let Some(parent) = Path::new(&game_html_location).parent() {let _ = fs::create_dir_all(parent);}
                let _ = fs::write(&game_html_location, game_html.clone());
            
                let game_json: Value = if let Some(json_start) = game_html.find("{\"available_games\":") {
                    let text_after = &game_html[json_start..];
                    if let Some(json_end) = text_after.find("}}") {
                        let json_text = &text_after[..json_end + 2];
                        serde_json::from_str(json_text).unwrap_or(json!({}))
                    } else {json!({})}
                } else {json!({})};
            
                let l_url = "https:".to_owned() + &game_json.get(a_device.clone()).and_then(|device| device.get("server_url")).and_then(|server_url| server_url.as_str()).unwrap_or_default().replace("{QUEUE}", &Uuid::new_v4().to_string());
                let l_token = game_json.get("options").and_then(|options| options.get("token")).and_then(|token| token.as_str()).unwrap_or_default().to_string();
                let l_language = game_json.get("options").and_then(|options| options.get("lang")).and_then(|lang| lang.as_str()).unwrap_or_default().to_string();
            
                default_data.command = "login".to_string();
                default_data.game = Default::default();
                default_data.platform = Default::default();
                default_data.re_enter = false;
                default_data.prev_request_id = Default::default();
                default_data.request_id = Uuid::new_v4().to_string();
                default_data.wl = Default::default();
                default_data.session_id = Default::default();
                default_data.token = l_token.clone();
                default_data.language = l_language.clone();
                default_data.mode = "play".to_string();
                default_data.huid = Default::default();
                default_data.action = Default::default();
                default_data.set_denominator = 1;
                default_data.quick_spin = 2;
                default_data.sound = false;
                default_data.autogame = true;
                default_data.mobile = "0".to_string();
                default_data.portrait = false;
                default_data.prev_client_command_time = None;
                
                Ok(l_url)
            },
            Err(err) => {eprintln!("{}", err.to_string()); Err(err.into())},
        },
        Err(err) => {eprintln!("{}", err.to_string()); Err(err.into())},
    }
}

pub async fn launch_game_v2(start_url: &str, default_data: &mut GameData) -> Result<String> {
    let client = Client::builder().redirect(Policy::limited(10)).build()?;
    
    let resp = client.get(start_url).send().await?;
    let final_url = resp.url();
    let params: HashMap<_, _> = final_url.query_pairs().into_owned().collect();
    let token     = params.get("token").unwrap();
    let game_name = params.get("game").unwrap();
    let lang      = params.get("lang").unwrap();
    let wl        = params.get("wl").unwrap();
    let exit_url  = params.get("exit_url").unwrap();
    let promo     = params.get("promo_widget").unwrap();
    let quickspin = params.get("quickspin").unwrap();
    let mobile    = params.get("mobile").map(String::as_str).unwrap_or("0");
    let currency  = params.get("currency").map(String::as_str).unwrap_or("FUN");
    let host = final_url.host_str().unwrap();
    let server_domain = host.replace("static-r2-sg-stage", "gsc-stage");
    let server_url = format!("https://{}/b/server", server_domain);

    let mut runner_url = Url::parse("https://static-r2-sg-stage.newreels.tech/aux/games/aux_b/runner/index.html")?;
    runner_url.query_pairs_mut()
        .append_pair("gameName", game_name)
        .append_pair("game", game_name)
        .append_pair("exit_url", exit_url)
        .append_pair("key", token)
        .append_pair("lang", lang)
        .append_pair("wl", wl)
        .append_pair("server_url", &server_url)
        .append_pair("currency", currency)
        .append_pair("platform", if mobile == "0" { "mob" } else { "web" })
        .append_pair("autoplay_mode", "advanced")
        .append_pair("promo_widget", promo)
        .append_pair("quickspin", quickspin)
        .append_pair("logo", "bng")
        .append_pair("show_currency", "1")
        .append_pair("in_game_lobby_url", "https://gsc-rc.newreels.tech/lobby")
        .append_pair("send_logout_onbeforeunload", "false");
    let _runner_html = client.get(runner_url).send().await?;

    default_data.huid = token.clone();
    default_data.token = token.clone();
    default_data.language = lang.clone();

        default_data.command = "login".to_string();
        default_data.game = game_name.clone();
        default_data.platform = if mobile == "0" { "mob".to_string()} else { "web".to_string()};
        default_data.re_enter = false;
        default_data.prev_request_id = Default::default();
        default_data.request_id = Uuid::new_v4().to_string();
        default_data.wl = wl.clone();
        default_data.session_id = Default::default();
        default_data.token = token.clone();
        default_data.language = lang.clone();
        default_data.mode = "play".to_string();
        default_data.huid = token.clone();
        default_data.action = Default::default();
        default_data.set_denominator = 1;
        default_data.quick_spin = 2;
        default_data.sound = false;
        default_data.autogame = true;
        default_data.mobile = "0".to_string();
        default_data.portrait = false;
        default_data.prev_client_command_time = None;

    Ok(server_url.clone())
}


async fn _send_request(client: &Client, url: &str, params: &HashMap<&str, &str>, body: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let res = client.post(url).query(params).json(body).send().await?.json::<Value>().await?;
    Ok(res)
}

pub async fn send_exec(a_game: &mut Game) {
    let max_retries = 10; 
    let mut attempts = 0;
    while attempts < max_retries {
        let start_time = Instant::now();
        match a_game.client.post(&a_game.request.url).query(&a_game.request.params).json(&a_game.request.body).send().await {
            Ok(response) => {
                attempts = 0;
                if response.status().is_success() {
                    a_game.data.prev_request_id = a_game.data.request_id.clone();
                    a_game.data.prev_client_command_time = Some(start_time.elapsed().as_millis() as i64);
                    a_game.response = response.json::<Value>().await.unwrap_or_default();
                    return;
                } else {
                    a_game.response = serde_json::to_value(json!({"error": response.status().to_string()})).unwrap_or_default();
                    print!("\x1B[1A\x1B[2K");
                    print!("\x1B[1A\x1B[2K");
                    eprintln!("\r\tServer error: {}", response.status());
                    print!("\x1B[K\t\n");
                    print!("\x1B[K\t\n");
                }
            }
            Err(err) => {
                a_game.response = serde_json::to_value(json!({"error": &err.to_string()})).unwrap_or_default();
                print!("\x1B[1A\x1B[2K");
                print!("\x1B[1A\x1B[2K");
                eprintln!("\r\tConnection error: {}. attempts {}/{}", err, attempts + 1, max_retries);
                print!("\x1B[K\t\n");
                print!("\x1B[K\t\n");
            }
        }
        attempts += 1;
        tokio::time::sleep(Duration::from_secs(3)).await; 
    }
    print!("\x1B[1A\x1B[2K");
    print!("\x1B[1A\x1B[2K");
    eprintln!("\r\tMaximum number of attempts exceeded!");
}