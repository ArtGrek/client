use serde_json::json;
use serde::{Deserialize, Serialize};
use std::{io::{self, Write}, collections::HashMap, };
use rand;
use tokio::time::{sleep, Duration};

use super::{Game, GameData, network::send_exec, storage::log_request_response};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Play {
    pub token: String,
    pub game: String,
    pub language: String,
    pub mode: String,
    pub fingerprint: String,
    pub context: Context,
    pub bet: Bet,
    #[serde(skip_serializing_if = "Option::is_none", rename = "buyFeature")]
    pub buy_feature: Option<BuyFeature>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Context {
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Bet {
    pub slot: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BuyFeature {
    pub cost: String,
    pub id: u32,
}

impl From<GameData> for Play {
    fn from(obj: GameData) -> Self {
        Play { 
            token: obj.session_id.clone(),
            game: "super-grand-link-express-hold-and-win".to_string(), 
            language: "en-GB".to_string(), 
            mode: "demo".to_string(), 
            fingerprint: obj.token.clone(), 
            context: Context::default(), 
            bet: Bet { 
                slot: "1.00".to_string() 
            }, 
            buy_feature: Some(BuyFeature {
                cost: vec!["70", "150"][obj.action.params.selected_mode.clone().unwrap_or_default().parse::<usize>().unwrap_or_default()-293].to_string(),
                id: obj.action.params.selected_mode.clone().unwrap_or_default().parse::<u32>().unwrap_or_default()
            })
        }
    }
}

pub async fn execute(a_game: &mut Game, must_delay: bool, delay: i64) {
    println!(" - {}", &a_game.transactions_file);
        send_exec("get_last_state", a_game).await;
        let l_session_id = a_game.response.get("user").and_then(|user| user.get("token")).and_then(|token| token.as_str()).unwrap_or_default().to_string();
        let l_huid = a_game.response.get("user").and_then(|user| user.get("id")).and_then(|id| id.as_str()).unwrap_or_default().to_string();
        a_game.data.session_id = l_session_id.clone();
        a_game.data.huid = l_huid.clone();
        let mut l_balance: f64 = a_game.response.get("user").and_then(|user| user.get("balance")).and_then(|balance| balance.get("cash")).and_then(|cash| cash.as_str()?.parse::<f64>().ok()).unwrap_or(0.0);
        let mut l_request_count = 0;
        println!("\tBalance: {:.2}", (l_balance as f64) / 100.0);
        println!("\tRequests count: {}", l_request_count);
        loop {
            next_body_exec(a_game).await;
            send_exec("api", a_game).await;
            let _ = log_request_response(&a_game.transactions_file, &json!({"in": a_game.request.body,"out": a_game.response}));
            if !a_game.response.get("user").and_then(|user| user.get("notifications")).and_then(|notifications| notifications.as_array().cloned()).unwrap_or_default().is_empty()
            && (a_game.data.action.params.selected_mode.is_none() || a_game.params.buy_bonus_only) {
                eprintln!("\r\tRequests stoped couse API error: {:?}", a_game.response.get("user").and_then(|user| user.get("notifications")).and_then(|notifications| notifications.as_array().cloned()).unwrap_or_default());
                break;
            }
            l_balance = a_game.response.get("user").and_then(|user| user.get("balance")).and_then(|balance| balance.get("cash")).and_then(|cash| cash.as_str()?.parse::<f64>().ok()).unwrap_or(0.0);
            l_request_count += 1;
            print!("\x1B[1A\x1B[2K");
            print!("\x1B[1A\x1B[2K");
            let _ = io::stdout().flush();
            print!("\x1B[K\tBalance: {:.2}\n", (l_balance as f64) / 100.0);
            print!("\x1B[K\tRequests count: {}\n", l_request_count);
            if must_delay {sleep(Duration::from_millis(rand::random_range(500..=delay as u64))).await;}
        }
}

fn set_spin(a_game: &mut Game) {
    a_game.data.seq += 1;
    a_game.data.action.params.bet_per_line = Some(a_game.params.bet_per_line);
    a_game.data.action.params.lines = Some(a_game.params.line);
    a_game.data.action.params.bet_factor = None;
    a_game.data.action.params.selected_mode = None;
    let mut play = Play::from(a_game.data.clone());
    play.buy_feature = None;
    a_game.request.body = serde_json::to_value(&play).unwrap_or_default();
}

fn set_buy_spin(a_game: &mut Game) {
    a_game.data.seq += 1;
    a_game.data.action.params.bet_per_line = Some(a_game.params.bet_per_line);
    a_game.data.action.params.lines = Some(a_game.params.line);
    a_game.data.action.params.bet_factor = Some(a_game.params.bet_factor);
    a_game.data.action.params.selected_mode = Some(a_game.params.selected_mode.to_string());
    a_game.request.body = serde_json::to_value(&Play::from(a_game.data.clone())).unwrap_or_default();
}

async fn next_body_exec(a_game: &mut Game) {
    if a_game.params.can_buy_bonus && a_game.params.buy_bonus_only {
        set_buy_spin(a_game);
    } else {
        set_spin(a_game);
    }
}

