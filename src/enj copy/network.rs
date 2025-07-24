use serde_json::{json, Value};
use std::path::Path;
use std::fs;
use std::process::Command;
use tokio;
use tokio::time::Duration;
use tokio::time::Instant;
use super::{Game, GameData};
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use regex::Regex;

fn extract_values(html: &str) -> Option<(String, String, String, String, String)> {
    let queue_re = Regex::new(r#"queue: '\s*([^']+)'"#).unwrap();
    let token_re = Regex::new(r#"token: '\s*([^']+)'"#).unwrap();
    let lang_re = Regex::new(r#"lang: '\s*([^']*)'"#).unwrap();
    let wl_re = Regex::new(r#"wl : '\s*([^']+)'"#).unwrap();
    let server_url_re = Regex::new(r#"desktop: \{[^}]*server_url: '\s*([^']+)'"#).unwrap();
    let l_queue = queue_re.captures(html)?.get(1)?.as_str().to_string();
    let l_token = token_re.captures(html)?.get(1)?.as_str().to_string();
    let l_language = lang_re.captures(html)?.get(1)?.as_str().to_string();
    let l_wl = wl_re.captures(html)?.get(1)?.as_str().to_string();
    let l_server_url = server_url_re.captures(html)?.get(1)?.as_str().to_string();
    Some((l_queue, l_token, l_language, l_wl, l_server_url))
}


pub fn run_playwright_scrape(node_path: &str,script_path: &str,url: &str,save_dir: &str, a_transactions_location: &str) -> std::io::Result<(String, String, String)> {
    let status = Command::new(node_path).arg(script_path).arg(url).arg(save_dir).arg(a_transactions_location).status()?;
    if !status.success() {return Err(std::io::Error::new(std::io::ErrorKind::Other,"playwright_scrape.js завершился с ошибкой",));}
    let referer_path = Path::new(save_dir).join("first.html");
    let html_path = Path::new(save_dir).join("page.html");
    let cookies_path = Path::new(save_dir).join("cookies.json");
    let referer_page = fs::read_to_string(&referer_path)?;
    let re = Regex::new(r#"src="([^"]+)""#).unwrap();
    let referer = re.captures(&referer_page).and_then(|cap| cap.get(1)).map(|m| m.as_str().to_string()).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Не удалось найти src в iframe"))?;
    let html = fs::read_to_string(&html_path)?;
    let cookies = fs::read_to_string(&cookies_path)?;
    Ok((html, cookies, referer))
}

pub async fn launch_game_v1(a_launch_url: String, a_location: String, a_game_name: String, _a_device: String, a_transactions_location: String, default_data: &mut GameData) -> Result<(String, String, String)> {

    let ts = Utc::now().timestamp_millis().to_string();
    let save_dir = &(a_location.to_owned() + &a_game_name.clone() + "/cloudflare/" + &ts.clone() + "/");
    fs::create_dir_all(&save_dir)?;
    let (game_html, cookies, referer) = run_playwright_scrape(
        "node",
        &(a_location.to_owned() + &a_game_name.clone() + "/cloudflare/playwright_scrape.js"),
        &a_launch_url,
        save_dir,
        &a_transactions_location
    )?;
    let (l_token, l_language, l_wl, l_server_url) = if let Some((l_queue, l_token, mut l_language, l_wl, mut l_server_url)) = extract_values(&game_html) {
        l_server_url = l_server_url.replace("{QUEUE}", &l_queue);
        if l_language.is_empty() {l_language = "en".to_string()}
        (l_token, l_language, l_wl, l_server_url)
    } else {
        println!("Не удалось извлечь все значения");
        Default::default()
    };

    let l_session_id = fs::read_to_string(&(a_location.to_owned() + &a_game_name.clone() + "/cloudflare/" + &ts.clone() + "/session_id.txt"))?;
println!("l_session_id {:?}", l_session_id);
    let l_huid = fs::read_to_string(&(a_location.to_owned() + &a_game_name.clone() + "/cloudflare/" + &ts.clone() + "/huid.txt"))?;
println!("l_huid {:?}", l_huid);
    //let l_prev_request_id = fs::read_to_string(&(a_location.to_owned() + &a_game_name.clone() + "/cloudflare/" + &ts.clone() + "/request_id.txt"))?;
//println!("l_prev_request_id {:?}", l_prev_request_id);

    default_data.command = "login".to_string();
    default_data.game = Default::default();
    default_data.platform = Default::default();
    default_data.re_enter = false;
    //default_data.prev_request_id = l_prev_request_id;
    default_data.request_id = Uuid::new_v4().to_string();
    default_data.wl = l_wl;
    default_data.session_id = l_session_id;
    default_data.token = l_token.clone();
    default_data.language = l_language.clone();
    default_data.mode = "play".to_string();
    default_data.huid = l_huid;
    default_data.action = Default::default();
    //default_data.set_denominator = 1;
    default_data.quick_spin = 1;
    default_data.sound = false;
    default_data.autogame = false;
    default_data.mobile = false;
    default_data.portrait = false;
    //default_data.prev_client_command_time = None;
    
    Ok((l_server_url, cookies, referer))
}

pub async fn send_exec(a_game: &mut Game) {
    let max_retries = 10; 
    let mut attempts = 0;
    while attempts < max_retries {
        let _start_time = Instant::now();
        //let body_str = serde_json::to_string(&a_game.request.body).unwrap();
println!("a_game.request {:?}", a_game.request);
        match a_game.client.post(&a_game.request.url)
            .query(&a_game.request.params)
            //.body(body_str)
            .json(&a_game.request.body)
            //.header("Referer", &a_game.request.referer)
            //.header("Accept", "application/json")
            //.header("Content-Type", "text/plain;charset=UTF-8")
            //.header("Accept", "*/*")
            //.header("Accept-Language", "ru-RU,ru;q=0.9,en-US;q=0.8,en;q=0.7")
            //.header("Priority", "u=1, i")
            //.header("Sec-CH-UA", r#""Google Chrome";v="135", "Not-A.Brand";v="8", "Chromium";v="135""#)
            //.header("Sec-CH-UA-Mobile", "?0")
            //.header("Sec-CH-UA-Platform", "\"Windows\"")
            //.header("Sec-Fetch-Dest", "empty")
            //.header("Sec-Fetch-Mode", "cors")
            //.header("Sec-Fetch-Site", "cross-origin")
            //.header("Sec-Fetch-Storage-Access", "active")
            //.header("Origin", "https://client.prod-eg.live")
            //.header("Referer", "https://client.prod-eg.live/")
            .send().await {
            Ok(response) => {
                attempts = 0;
                if response.status().is_success() {
                    //a_game.data.prev_request_id = a_game.data.request_id.clone();
                    //a_game.data.prev_client_command_time = Some(start_time.elapsed().as_millis() as i64);
                    a_game.response = response.json::<Value>().await.unwrap_or_default();
                    return;
                } else {
                    a_game.response = serde_json::to_value(json!({"error": response.status().to_string()})).unwrap_or_default();
                    //print!("\x1B[1A\x1B[2K");
                    //print!("\x1B[1A\x1B[2K");
                    eprintln!("\r\tServer error: {}", response.status());
                    //print!("\x1B[K\t\n");
                    //print!("\x1B[K\t\n");
                }
            }
            Err(err) => {
                a_game.response = serde_json::to_value(json!({"error": &err.to_string()})).unwrap_or_default();
                //print!("\x1B[1A\x1B[2K");
                //print!("\x1B[1A\x1B[2K");
                eprintln!("\r\tConnection error: {}. attempts {}/{}", err, attempts + 1, max_retries);
                //print!("\x1B[K\t\n");
                //print!("\x1B[K\t\n");
            }
        }
println!("a_game.request {:?}", a_game.response);
        attempts += 1;
        tokio::time::sleep(Duration::from_secs(3)).await; 
    }
    //print!("\x1B[1A\x1B[2K");
    //print!("\x1B[1A\x1B[2K");
    eprintln!("\r\tMaximum number of attempts exceeded!");
}
