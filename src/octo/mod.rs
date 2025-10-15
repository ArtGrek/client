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
    let mut child = Command::new("node")
        .arg(&(a_location.to_owned() + &a_game_name.clone() + "/cloudflare/playwright_ws.js"))
        .arg(&l_launch_url)
        .arg(&temp_data_dir)
        .arg(&l_transactions_file)
        .arg(&l_port.to_string())
        .spawn()?;

    /*
    let prefix = a_location.to_owned() + &a_game_name.clone() + "/cloudflare/" + &ts + "/";
    if let Err(e) = wait_for_file(&(prefix.clone() + "token.txt"), 30).await {eprintln!("wait for token error: {e}");}
    let l_token = match fs::read_to_string(&(prefix.clone() + "token.txt")) {Ok(t) => t, Err(e) => {eprintln!("read token error: {e}");String::new()}};
    if let Err(e) = wait_for_file(&(prefix.clone() + "language.txt"), 30).await {eprintln!("wait for language error: {e}");}
    let l_language = match fs::read_to_string(&(prefix.clone() + "language.txt")) {Ok(t) => t, Err(e) => {eprintln!("read language error: {e}");String::new()}};
    if let Err(e) = wait_for_file(&(prefix.clone() + "wl.txt"), 30).await {eprintln!("wait for wl error: {e}");}
    let l_wl = match fs::read_to_string(&(prefix.clone() + "wl.txt")) {Ok(t) => t, Err(e) => {eprintln!("read wl error: {e}");String::new()}};
    if let Err(e) = wait_for_file(&(prefix.clone() + "url.txt"), 30).await {eprintln!("wait for url error: {e}");}
    let l_url = match fs::read_to_string(&(prefix.clone() + "url.txt")) {Ok(t) => t, Err(e) => {eprintln!("read url error: {e}");String::new()}};
    */

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
        "hold_and_win" => {hold_and_win::execute(&mut game, must_delay, delay).await;}
        _ => {eprintln!("\r\tGame not implement");}
    };
    let _ = child.kill();

    Ok(())
}