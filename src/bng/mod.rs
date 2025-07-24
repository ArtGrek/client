//use serde_json::{json, Value};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use uuid::Uuid;
use reqwest::Client;
use std::error::Error;
mod storage;
mod network;
mod china_festival;

#[derive(Debug, Default, Clone)]
pub struct Game {
    //pub name: String,
    pub params: GameParams,
    pub data: GameData,
    pub transactions_location: String,
    pub client: Client,
    pub request: Request,
    pub response:Value,
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
    pub selected_modes: Vec<i64>,
    pub selected_mode: i64,
    pub can_buy_bonus: bool,
    pub buy_bonus_only: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameData {
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
    pub mobile: String,
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
    let game_config: Value = serde_json::from_str(&(fs::read_to_string("./".to_owned() + &a_game_name + ".json").unwrap_or_default())).unwrap_or_default();
    let launch_vertion = game_config.get("launch_vertion").and_then(|v| {v.as_i64()}).unwrap_or(1);
    let l_launch_url = game_config.get("launch_url").and_then(|v| v.as_str()).unwrap_or("https://bng.games/play/{QUEUE}/?lang=en").to_string().replace("{QUEUE}", &a_game_name);
    let l_device = game_config.get("device").and_then(|v| v.as_str()).unwrap_or("desktop").to_string();
    let l_bets: Vec<i64> = game_config.get("bets").and_then(|v| v.as_array()).unwrap_or(&vec![]).iter().filter_map(|v| v.as_i64()).collect();
    let l_bet_per_line = game_config.get("bet_per_line").and_then(|v| {if l_bets.contains(&v.as_i64().unwrap_or_default()) {v.as_i64()} else {l_bets.get(0).cloned()}}).unwrap_or(1);
    let l_lines: Vec<i64> = game_config.get("lines").and_then(|v| v.as_array()).unwrap_or(&vec![]).iter().filter_map(|v| v.as_i64()).collect();
    let l_line = game_config.get("line").and_then(|v| {if l_lines.contains(&v.as_i64().unwrap_or_default()) {v.as_i64()} else {l_lines.get(0).cloned()}}).unwrap_or(1);
    let l_bet_factors: Vec<i64> = game_config.get("bet_factors").and_then(|v| v.as_array()).unwrap_or(&vec![]).iter().filter_map(|v| v.as_i64()).collect();
    let l_bet_factor = game_config.get("bet_factor").and_then(|v| {if l_bet_factors.contains(&v.as_i64().unwrap_or_default()) {v.as_i64()} else {l_bet_factors.get(0).cloned()}}).unwrap_or(20);
    let l_selected_modes: Vec<i64> = game_config.get("selected_modes").and_then(|v| v.as_array()).unwrap_or(&vec![]).iter().filter_map(|v| v.as_i64()).collect();
    let l_selected_mode = game_config.get("selected_mode").and_then(|v| {if l_selected_modes.contains(&v.as_i64().unwrap_or_default()) {v.as_i64()} else {l_selected_modes.get(0).cloned()}}).unwrap_or(1);
    let l_can_buy_bonus = game_config.get("can_buy_bonus").and_then(|v| v.as_bool()).unwrap_or(false);
    let l_buy_bonus_only = game_config.get("buy_bonus_only").and_then(|v| v.as_bool()).unwrap_or(false);
    
    let l_transactions_location = if l_buy_bonus_only {a_location.to_owned() + &a_game_name.clone() + "/transactions/buy_" + &l_selected_mode.to_string() + "_bet_" + &l_bet_per_line.to_string() + "_line_" + &l_line.to_string() + "_" + &Uuid::new_v4().to_string().replace("-", "") + ".json"}
    else {a_location.to_owned() + &a_game_name.clone() + "/transactions/bet_" + &l_bet_per_line.to_string() + "_line_" + &l_line.to_string() + "_" + &Uuid::new_v4().to_string().replace("-", "") + ".json"};
    if let Some(parent) = Path::new(&l_transactions_location).parent() {let _ = fs::create_dir_all(parent);}

    let mut default_data: GameData = Default::default();
    let l_url = match launch_vertion {
        1 => {network::launch_game_v1(l_launch_url.clone(), a_location.clone(), a_game_name.clone(), l_device.clone(), &mut default_data).await?},
        2 => {network::launch_game_v2(&l_launch_url, &mut default_data).await?},
        _ => {network::launch_game_v1(l_launch_url.clone(), a_location.clone(), a_game_name.clone(), l_device.clone(), &mut default_data).await?}
    };

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
        data: default_data,
        transactions_location: l_transactions_location.clone(), 
        client: Client::new(), 
        request: Request {
            url: l_url.clone(),
            params: HashMap::new() ,
            body: Default::default(),
        },
        response: Default::default()
    };
    
    match a_game_name.as_str() {
        "china_festival" => {china_festival::execute(&mut game, launch_vertion, must_delay, delay).await; Ok(())}
        "coin_lamp" => {china_festival::execute(&mut game, launch_vertion, must_delay, delay).await; Ok(())}
        "3_aztec_temples" => {china_festival::execute(&mut game, launch_vertion, must_delay, delay).await; Ok(())}
        _ => {eprintln!("\r\tGame not implement"); Ok(())}
    }

}