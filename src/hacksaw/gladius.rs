//china_festival, coin_lamp, 3_aztec_temples
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::{io::{self, Write}, /*process::exit*/};
use rand;
use tokio::time::{sleep, Duration};

use super::{Game, GameData, network::send_exec, storage::log_request_response};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Start {
    pub seq: i64,
    pub partner: String,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    pub currency: String,
    #[serde(rename = "languageCode")]
    pub language_code: String,
    pub mode: i32,
    pub branding: String,
    pub channel: i32,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    pub token: String,
}

impl From<GameData> for Start {
    fn from(obj: GameData) -> Self {
        Start {
            seq: obj.seq,
            partner: "demo".to_string(),
            game_id: 1807,
            game_version: "1.15.2".to_string(),
            currency: "EUR".to_string(),
            language_code: "en-us".to_string(),
            mode: 2,
            branding: "default".to_string(),
            channel: 1,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".to_string(),
            token: "demo".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Sync {
    #[serde(rename = "sessionUuid")]
    pub session_uuid: String,
}

impl From<GameData> for Sync {
    fn from(_obj: GameData) -> Self {
        Sync {
            session_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Play {
    pub seq: i64,
    #[serde(rename = "sessionUuid")]
    pub session_uuid: String,
    pub bets: Vec<Bet>,
    #[serde(rename = "offerId")]
    pub offer_id: Option<String>,
    #[serde(rename = "promotionId")]
    pub promotion_id: Option<String>,
    pub autoplay: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Bet {
    #[serde(rename = "betAmount")]
    pub bet_amount: String,
    #[serde(rename = "buyBonus", skip_serializing_if = "Option::is_none")]
    pub buy_bonus: Option<String>,
}

impl From<GameData> for Play {
    fn from(obj: GameData) -> Self {
        Play {
            seq: obj.seq,
            session_uuid: obj.session_id.clone(),
            bets: vec![Bet {
                bet_amount: obj.action.params.bet_per_line.unwrap_or_default().to_string(),
                buy_bonus: obj.action.params.selected_mode.clone(), 
            }],
            offer_id: None,
            promotion_id: None,
            autoplay: obj.autogame.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Collect {
    pub seq: i64,
    #[serde(rename = "sessionUuid")]
    pub session_uuid: String,
    #[serde(rename = "roundId")]
    pub round_id: String,
    #[serde(rename = "continueInstructions")]
    pub continue_instructions: ContinueInstructions,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ContinueInstructions {
    #[serde(rename = "action")]
    pub action: String,
}

impl From<GameData> for Collect {
    fn from(obj: GameData) -> Self {
        Collect {
            seq: obj.seq,
            session_uuid: obj.session_id.clone(),
            round_id: obj.round_id.clone(),
            continue_instructions: ContinueInstructions {
                action: "win_presentation_complete".to_string(),
            },
        }
    }
}

pub async fn execute(a_game: &mut Game, must_delay: bool, delay: i64) {
    println!(" - {}", &a_game.transactions_file);
        send_exec("get_last_state", a_game).await;
        let l_session_id = a_game.response.get("sessionUuid").and_then(|session_id| session_id.as_str()).unwrap_or_default().to_string();
        let l_huid = a_game.response.get("playerId").and_then(|huid| huid.as_str()).unwrap_or_default().to_string();
        a_game.data.session_id = l_session_id.clone();
        a_game.data.huid = l_huid.clone();
        a_game.data.autogame = true;
        let mut l_balance: i64 = a_game.response.get("accountBalance").and_then(|user| user.get("balance")).and_then(|balance| balance.as_str()?.parse::<i64>().ok()).unwrap_or(0);
        let mut l_request_count = 0;
        println!("\tBalance: {:.2}", (l_balance as f64) / 100.0);
        println!("\tRequests count: {}", l_request_count);
        let mut l_game: Game;
        loop {
            l_game = a_game.clone();
            next_body_exec(a_game);
            send_exec("api", a_game).await;
            let _ = log_request_response(&a_game.transactions_file, &json!({"in": a_game.request.body,"out": a_game.response}));
            if a_game.response.get("statusCode").and_then(|code| code.as_i64()) != Some(0)
            && (a_game.data.action.params.selected_mode.is_none() || a_game.params.buy_bonus_only) {
                eprintln!("\r\tRequests stoped couse API error: {}", a_game.response.get("statusCode").and_then(|code| {code.as_i64()}).unwrap_or_default());
                break;
            }
            l_balance = a_game.response.get("accountBalance").and_then(|user| user.get("balance")).and_then(|balance| balance.as_str()?.parse::<i64>().ok()).unwrap_or(0);
            l_request_count += 1;
            print!("\x1B[1A\x1B[2K");
            print!("\x1B[1A\x1B[2K");
            let _ = io::stdout().flush();
            print!("\x1B[K\tBalance: {:.2}\n", (l_balance as f64) / 100.0);
            print!("\x1B[K\tRequests count: {}\n", l_request_count);
            if must_delay {sleep(Duration::from_millis(rand::random_range(500..=delay as u64))).await;}
            if a_game.request.params.get("gsc") == Some(&"sync".to_string()) {*a_game = l_game.clone()}
        }
}

fn set_start(a_game: &mut Game) {
    a_game.data.seq += 1;
    a_game.request.body = serde_json::to_value(&Start::from(a_game.data.clone())).unwrap_or_default();
}

fn set_spin(a_game: &mut Game) {
    a_game.data.seq += 1;
    a_game.data.action.params.bet_per_line = Some(a_game.params.bet_per_line);
    a_game.data.action.params.lines = Some(a_game.params.line);
    a_game.data.action.params.bet_factor = None;
    a_game.data.action.params.selected_mode = None;
    a_game.request.body = serde_json::to_value(&Play::from(a_game.data.clone())).unwrap_or_default();
}

fn set_buy_spin(a_game: &mut Game) {
    a_game.data.seq += 1;
    a_game.data.action.params.bet_per_line = Some(a_game.params.bet_per_line);
    a_game.data.action.params.lines = Some(a_game.params.line);
    a_game.data.action.params.bet_factor = Some(a_game.params.bet_factor);
    a_game.data.action.params.selected_mode = Some(a_game.params.selected_mode.to_string());
    a_game.request.body = serde_json::to_value(&Play::from(a_game.data.clone())).unwrap_or_default();
}

fn set_collect(a_game: &mut Game) {
    a_game.data.seq += 1;
    a_game.data.round_id = a_game.response.get("round").and_then(|round| {round.get("roundId")}).and_then(|round_id| {round_id.as_str()}).map(|s| s.to_string()).unwrap_or_default();
    a_game.request.body = serde_json::to_value(&Collect::from(a_game.data.clone())).unwrap_or_default();
}

fn next_body_exec(a_game: &mut Game) {
    if rand::random_range(0..1_000) == 0 {
        set_start(a_game);
    } else {
        if let Some(round) = a_game.response.get("round") {
            if round.get("status").and_then(|status| {status.as_str()}) == Some("wfwpc") 
            && round.get("events").and_then(|events| {events.as_array()}).map(|arr| arr.len()) > Some(1) {
                set_collect(a_game);
            } else if a_game.params.can_buy_bonus && a_game.params.buy_bonus_only {set_buy_spin(a_game);} else {set_spin(a_game);}
        } else {if a_game.params.can_buy_bonus && a_game.params.buy_bonus_only {set_buy_spin(a_game);} else {set_spin(a_game);}}
    }
}

