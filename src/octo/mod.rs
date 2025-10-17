use serde_json::Value;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use uuid::Uuid;
use std::error::Error;
use std::process::Command;
use anyhow::Result;
mod storage;
mod network;
mod hold_and_win;
use url::Url;
use chrono::Utc;
use rand;
use std::net::TcpListener;
use tokio::signal;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::SinkExt;

#[derive(Debug, Clone)]
pub struct Game {
    //pub name: String,
    pub params: GameParams,
    pub data: GameData,
    pub transactions_file: String,
    pub ws_client: Url,
    pub request: Request,
    pub response:Value,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            params: Default::default(),
            data: Default::default(),
            transactions_file: Default::default(),
            ws_client: Url::parse("ws://localhost:3000").unwrap(),
            request: Default::default(),
            response: Default::default(), 
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Request {
    pub url: String,
    pub params: HashMap<String, String>,
    pub body: Value,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameParams {
    //pub launch_url: String,
    //pub device: String,
    //pub bets: Vec<i64>,
    pub bet_per_line: i64,
    //pub lines: Vec<i64>,
    pub line: i64,
    //pub bet_factors: Vec<i64>,
    pub bet_factor: i64,
    pub selected_modes: Vec<String>,
    pub selected_mode: String,
    pub can_buy_bonus: bool,
    pub buy_bonus_only: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameData {
    pub seq: i64,
    pub round_id: String,
    pub command: String,
    pub game: String,
    pub platform: String,
    #[serde(rename = "re-enter")]
    pub re_enter: bool,
    pub prev_request_id: String,
    pub request_id: String,
    pub wl: String,
    pub session_id: String,
    pub token: String,
    pub language: String,
    pub mode: String,
    pub huid: String,
    pub action: Action,
    pub set_denominator: i64,
    pub quick_spin: i64,
    pub sound: bool,
    pub autogame: bool,
    pub mobile: bool,
    pub portrait: bool,
    pub prev_client_command_time: Option<i64>
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Action {
    pub name: String,
    pub params: Params
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_per_line: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_factor: Option<i64>
}

pub async fn execute(a_game_name: String, a_location: String, must_delay: bool, delay: i64) -> Result<(), Box<dyn Error>> {
    //game config
    let game_config: Value = serde_json::from_str(&(fs::read_to_string("./configs/".to_owned() + &a_game_name + ".json").unwrap_or_default())).unwrap_or_default();
    let _launch_vertion = game_config.get("launch_vertion").and_then(|v| {v.as_i64()}).unwrap_or(1);
    let l_launch_url = game_config.get("launch_url").and_then(|v| v.as_str()).unwrap_or("https://demo.enjoygaming.com/start-game/{QUEUE}").to_string().replace("{QUEUE}", &a_game_name);
    let _l_device = game_config.get("device").and_then(|v| v.as_str()).unwrap_or("desktop").to_string();
    let l_bets: Vec<i64> = game_config.get("bets").and_then(|v| v.as_array()).unwrap_or(&vec![]).iter().filter_map(|v| v.as_i64()).collect();
    let l_bet_per_line = game_config.get("bet_per_line").and_then(|v| {if l_bets.contains(&v.as_i64().unwrap_or_default()) {v.as_i64()} else {l_bets.get(0).cloned()}}).unwrap_or(1);
    let l_lines: Vec<i64> = game_config.get("lines").and_then(|v| v.as_array()).unwrap_or(&vec![]).iter().filter_map(|v| v.as_i64()).collect();
    let l_line = game_config.get("line").and_then(|v| {if l_lines.contains(&v.as_i64().unwrap_or_default()) {v.as_i64()} else {l_lines.get(0).cloned()}}).unwrap_or(1);
    let l_bet_factors: Vec<i64> = game_config.get("bet_factors").and_then(|v| v.as_array()).unwrap_or(&vec![]).iter().filter_map(|v| v.as_i64()).collect();
    let l_bet_factor = game_config.get("bet_factor").and_then(|v| {if l_bet_factors.contains(&v.as_i64().unwrap_or_default()) {v.as_i64()} else {l_bet_factors.get(0).cloned()}}).unwrap_or(20);
    let l_selected_modes: Vec<String> = game_config.get("selected_modes").and_then(|v| v.as_array()).unwrap_or(&vec![]).iter().filter_map(|v| Some(v.as_str()?.to_string())).collect();
    let l_selected_mode: String = game_config.get("selected_mode").and_then(|v| v.as_str()).map(|s| s.to_string()).filter(|s| l_selected_modes.contains(s)).unwrap_or_else(|| l_selected_modes.get(0).cloned().unwrap_or_else(|| "mod_bonus".to_string()));
    let l_can_buy_bonus = game_config.get("can_buy_bonus").and_then(|v| v.as_bool()).unwrap_or(false);
    let l_buy_bonus_only = game_config.get("buy_bonus_only").and_then(|v| v.as_bool()).unwrap_or(false);
    
    let l_transactions_file = if l_buy_bonus_only {a_location.to_owned() + &a_game_name.clone() + "/transactions/buy_" + &l_selected_mode.to_string() + "_bet_" + &l_bet_per_line.to_string() + "_line_" + &l_line.to_string() + "_" + &Uuid::new_v4().to_string().replace("-", "") + ".json"}
    else {a_location.to_owned() + &a_game_name.clone() + "/transactions/bet_" + &l_bet_per_line.to_string() + "_line_" + &l_line.to_string() + "_" + &Uuid::new_v4().to_string().replace("-", "") + ".json"};
    if let Some(parent) = Path::new(&l_transactions_file).parent() {let _ = fs::create_dir_all(parent);}

    // ===== общие переменные для доступа из хука =====
    let chrome_process: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
    let node_process: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));

    // chrome
    let chrome_path = r#"C:\Program Files\Google\Chrome\Application\chrome.exe"#;
    /*let chrome_args = [
        "--remote-debugging-port=9222",
        "--user-data-dir=C:\\ChromeDebug",
    ];*/
    let ts = Utc::now().timestamp_millis();
    let user_data_dir = format!("C:\\ChromeDebug_{}", ts);
    let chrome_args = [
        "--remote-debugging-port=9222",
        &format!("--user-data-dir={}", user_data_dir),
    ];
    let chrome_child = Command::new(chrome_path)
        .args(&chrome_args)
        .spawn()?;

    println!("Run Chrome with PID {:?}", chrome_child.id());
    *chrome_process.lock().unwrap() = Some(chrome_child);
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // script
    let mut l_port = 0;
    for _ in 0..10 {
        let try_port = 3001 + rand::random_range(0..1000);
        if let Ok(_listener) = TcpListener::bind(("127.0.0.1", try_port)) {
            l_port = try_port;
            break;
        }
    }
    if l_port == 0 {return Err("\r\tCannot find port".into());}
    let ts = Utc::now().timestamp_millis().to_string();
    let temp_data_dir = a_location.to_owned() + &a_game_name.clone() + "/cloudflare/" + &ts.clone() + "/";
    fs::create_dir_all(&temp_data_dir)?;
    let child = Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("node")
        .arg(&(a_location.to_owned() + &a_game_name.clone() + "/client/playwright_ws.js"))
        .arg(&l_launch_url)
        .arg(&temp_data_dir)
        .arg(&l_transactions_file)
        .arg(&l_port.to_string())
        .spawn()?;
    println!("Run script with PID {:?}", child.id());
    *node_process.lock().unwrap() = Some(child);
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // ===== общие переменные для доступа из хука =====
    let chrome_ref = chrome_process.clone();
    let node_ref = node_process.clone();
    let ws_port = l_port;
    let user_data_dir_ref = user_data_dir.clone();
    // ===== хук Ctrl+C =====
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            eprintln!("[SIGNAL] Ctrl+C detected — shutting down...");
            let ws_url = format!("ws://localhost:{ws_port}");
            if let Ok((mut ws_stream, _)) = tokio_tungstenite::connect_async(&ws_url).await {
                let _ = ws_stream.send(Message::Text(r#"{"type":"shutdown"}"#.into())).await;
                eprintln!("→ Sent shutdown command to Node script");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            } else {
                eprintln!("→ Failed to connect to Node WS (already closed?)");
            }
            if let Some(mut node_child) = node_ref.lock().unwrap().take() {
                let pid = node_child.id();
                eprintln!("→ Killing Node (PID: {pid})");
                let _ = node_child.kill();
                let _ = node_child.wait();
            }
            if let Some(mut chrome_child) = chrome_ref.lock().unwrap().take() {
                let pid = chrome_child.id();
                eprintln!("→ Killing Chrome (PID: {pid})");
                let _ = chrome_child.kill();
                let _ = chrome_child.wait();
            }
            let _ = std::fs::remove_dir_all(&user_data_dir_ref);
            std::process::exit(0);
        }
    });


    let mut game: Game = Game {
        //name: a_game_name.clone(), 
        params: GameParams {
            bet_per_line: l_bet_per_line.clone(),
            line: l_line.clone(),
            bet_factor: l_bet_factor.clone(),
            selected_modes: l_selected_modes.clone(),
            selected_mode: l_selected_mode.clone(),
            can_buy_bonus: l_can_buy_bonus.clone(),
            buy_bonus_only: l_buy_bonus_only.clone(),
        }, 
        data: GameData { 
            seq: 1,
            round_id: Default::default(), 
            command: "login".to_string(), 
            game: Default::default(), 
            platform: Default::default(), 
            re_enter: false, 
            prev_request_id: Default::default(), 
            request_id: Uuid::new_v4().to_string(), 
            wl: Default::default(), 
            session_id: Default::default(), 
            token: Uuid::new_v4().to_string(), 
            language: Default::default(), 
            mode: "play".to_string(), 
            huid: Default::default(),
            action: Default::default(), 
            set_denominator: Default::default(), 
            quick_spin: 2, 
            sound: false, 
            autogame: false, 
            mobile: false, 
            portrait: false, 
            prev_client_command_time: None 
        },
        transactions_file: l_transactions_file.clone(),
        ws_client: Url::parse(&format!("ws://localhost:{}", l_port)).expect(&format!("set ws://localhost:{} failed", l_port)),
        request: Request {
            url: "https://lobby-api.octoplay.com/platform/public/bet".to_string(),
            params: HashMap::new() ,
            body: Default::default(),
        },
        response: Default::default(),
    };

    
    println!("PID: {} Port: {}", std::process::id(), l_port);
    match a_game_name.as_str() {
        "hold_and_win" => {let _ = hold_and_win::execute(&mut game, must_delay, delay).await;}
        _ => {eprintln!("\r\tGame not implement");}
    };
    
    let _ = super::octo::network::send_exec("shutdown", &mut game).await;
    if let Some(mut child) = node_process.lock().unwrap().take() {
        let pid = child.id();
        println!("Stopping script with PID {pid}...");
        let _ = child.kill();
        let _ = child.wait();
    }
    if let Some(mut chrome_child) = chrome_process.lock().unwrap().take() {
        let pid = chrome_child.id();
        println!("Stopping Chrome with PID {pid}...");
        let _ = chrome_child.kill();
        let _ = chrome_child.wait();
    }
    let _ = std::fs::remove_dir_all(&user_data_dir);

    Ok(())
}