//china_festival, coin_lamp, 3_aztec_temples
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use std::{io::{self, Write}, /*process::exit*/};
use rand;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use super::{Game, GameData, Action, network::send_exec, storage::log_request_response};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct _LoginV1 {
    pub command: String,
    pub request_id: String,
    pub token: String,
    pub language: String
}

impl From<GameData> for _LoginV1 {
    fn from(obj: GameData) -> Self {
        _LoginV1 {
            command: obj.command.clone(),
            request_id: obj.request_id.clone(),
            token: obj.token.clone(),
            language: obj.language.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct _LoginV2 {
    pub command: String,
    pub game: String,
    pub platform: String,
    pub playerguid: String,
    #[serde(rename = "re-enter")]
    pub re_enter: bool,
    pub request_id: String,
    pub wl: String,
}

impl From<GameData> for _LoginV2 {
    fn from(obj: GameData) -> Self {
        _LoginV2 {
            command: obj.command.clone(),
            game: obj.game.clone(),
            platform: obj.platform.clone(),
            playerguid: obj.huid.clone(),
            re_enter: obj.re_enter.clone(),
            request_id: obj.request_id.clone(),
            wl: obj.wl.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Start {
    pub command: String,
    //pub prev_request_id: String,
    pub request_id: String,
    pub session_id: String,
    pub mode: String,
    pub huid: String,
    pub prev_client_command_time: i64
}

impl From<GameData> for Start {
    fn from(obj: GameData) -> Self {
        Start {
            command: obj.command.clone(),
            //prev_request_id: obj.prev_request_id.clone(),
            request_id: obj.request_id.clone(),
            session_id: obj.session_id.clone(),
            mode: obj.mode.clone(),
            huid: obj.huid.clone(),
            prev_client_command_time: obj.prev_client_command_time.unwrap_or(0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Sync {
    pub command: String,
    pub request_id: String,
    pub session_id: String,
}

impl From<GameData> for Sync {
    fn from(obj: GameData) -> Self {
        Sync {
            command: obj.command.clone(),
            request_id: obj.request_id.clone(),
            session_id: obj.session_id.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Play {
    pub command: String,
    //pub prev_request_id: String,
    pub request_id: String,
    pub session_id: String,
    pub action: Action,
    //pub set_denominator: i64,
    pub quick_spin: i64,
    pub sound: bool,
    pub autogame: bool,
    pub mobile: bool,
    pub portrait: bool,
    pub prev_client_command_time: i64
}

impl From<GameData> for Play {
    fn from(obj: GameData) -> Self {
        Play {
            command: obj.command.clone(),
            //prev_request_id: obj.prev_request_id.clone(),
            request_id: obj.request_id.clone(),
            session_id: obj.session_id.clone(),
            action: obj.action.clone(),
            //set_denominator: obj.set_denominator.clone(),
            quick_spin: obj.quick_spin.clone(),
            sound: obj.sound.clone(),
            autogame: obj.autogame.clone(),
            mobile: obj.mobile.clone(),
            portrait: obj.portrait.clone(),
            prev_client_command_time: obj.prev_client_command_time.unwrap_or(0),
        }
    }
}

struct Restart {
    pub buy_spin: bool,
    pub win: bool,
    pub bonus_init: bool,
    pub bonus_init_befor: bool,
    pub respin: bool,
    pub bonus_spins_stop: bool,
    pub bonus_spins_stop_befor: bool,
}

pub async fn execute(a_game: &mut Game, must_delay: bool, delay: i64) {
    println!(" - {}", &a_game.transactions_file);
        send_exec("get_last_state", a_game).await;
        //login
        /*a_game.request.params.clear();
        a_game.request.params.insert("gsc".to_string(), "login".to_string());
        a_game.request.body = match launch_vertion {
            1 => {serde_json::to_value(&LoginV1::from(a_game.data.clone())).unwrap_or_default()},
            2 => {serde_json::to_value(&LoginV2::from(a_game.data.clone())).unwrap_or_default()},
            _ => {serde_json::to_value(&LoginV1::from(a_game.data.clone())).unwrap_or_default()}
        };
        send_exec("api", a_game).await;
        let _ = log_request_response(&a_game.transactions_file, &json!({"in": a_game.request.body,"out": a_game.response}));
        if a_game.response.get("status").and_then(|status| status.get("code")).and_then(|code| code.as_str()) != Some("OK") {
            eprintln!("\r\tRequests stoped couse API error: {}", a_game.response.get("status").and_then(|status| status.get("code")).and_then(|code| {code.as_str()}).unwrap_or("unknown"));
            exit(1);
        }*/
        let l_session_id = a_game.response.get("session_id").and_then(|session_id| session_id.as_str()).unwrap_or_default().to_string();
        let l_huid = a_game.response.get("user").and_then(|user| user.get("huid")).and_then(|huid| huid.as_str()).unwrap_or_default().to_string();
        a_game.data.session_id = l_session_id.clone();
        a_game.data.huid = l_huid.clone();
        //start
        /*set_start(a_game);
        send_exec("api", a_game).await;
        let _ = log_request_response(&a_game.transactions_file, &json!({"in": a_game.request.body,"out": a_game.response}));
        if a_game.response.get("status").and_then(|status| status.get("code")).and_then(|code| code.as_str()) != Some("OK") {
            eprintln!("\r\tRequests stoped couse API error: {}", a_game.response.get("status").and_then(|status| status.get("code")).and_then(|code| {code.as_str()}).unwrap_or("unknown"));
            exit(1);
        }*/
        let mut l_balance: i64 = a_game.response.get("user").and_then(|user| user.get("balance")).and_then(|balance| balance.as_i64()).unwrap_or(0);
        let mut l_request_count = 0;
        println!("\tBalance: {:.2}", (l_balance as f64) / 100.0);
        println!("\tRequests count: {}", l_request_count);
        let mut l_restart: Restart = Restart {buy_spin: (true), win: (true), bonus_init: (true), bonus_init_befor: (true), respin: (true), bonus_spins_stop: (true), bonus_spins_stop_befor: (true)};
        let mut l_game: Game;
        loop {
            l_game = a_game.clone();
            next_body_exec(a_game, &mut l_restart);
            send_exec("api", a_game).await;
            let _ = log_request_response(&a_game.transactions_file, &json!({"in": a_game.request.body,"out": a_game.response}));
            //if a_game.response.get("status").and_then(|status| status.get("code")).and_then(|code| code.as_str()) == Some("OK") {
            //    a_game.data.prev_request_id = a_game.data.request_id.clone();
            //}
            if a_game.response.get("status").and_then(|status| status.get("code")).and_then(|code| code.as_str()) != Some("OK")
            && (a_game.request.body.get("action").and_then(|action| action.get("name")).and_then(|name| name.as_str()) != Some("buy_spin") || a_game.params.buy_bonus_only) {
                eprintln!("\r\tRequests stoped couse API error: {}", a_game.response.get("status").and_then(|status| status.get("code")).and_then(|code| {code.as_str()}).unwrap_or("unknown"));
                break;
            }
            l_balance = a_game.response.get("user").and_then(|user| user.get("balance")).and_then(|balance| balance.as_i64()).unwrap_or(0);
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
    a_game.data.command = "start".to_string();
    a_game.data.request_id = Uuid::new_v4().simple().to_string();
    a_game.request.params.clear();
    a_game.request.params.insert("gsc".to_string(), "start".to_string());
    a_game.request.body = serde_json::to_value(&Start::from(a_game.data.clone())).unwrap_or_default();
}

fn set_sync(a_game: &mut Game) {
    a_game.data.command = "sync".to_string();
    a_game.data.request_id = Uuid::new_v4().simple().to_string();
    a_game.request.params.clear();
    a_game.request.params.insert("gsc".to_string(), "sync".to_string());
    a_game.request.body = serde_json::to_value(&Sync::from(a_game.data.clone())).unwrap_or_default();
}

fn set_spin(a_game: &mut Game) {
    a_game.data.command = "play".to_string();
    a_game.data.request_id = Uuid::new_v4().simple().to_string();
    a_game.data.action.name = "spin".to_string();
    a_game.data.action.params.bet_per_line = Some(a_game.params.bet_per_line);
    a_game.data.action.params.lines = Some(a_game.params.line);
    a_game.data.action.params.bet_factor = None;
    a_game.data.action.params.selected_mode = None;
    a_game.request.params.clear();
    a_game.request.params.insert("gsc".to_string(), "play".to_string());
    a_game.request.body = serde_json::to_value(&Play::from(a_game.data.clone())).unwrap_or_default();
}

fn set_buy_spin(a_game: &mut Game) {
    a_game.data.command = "play".to_string();
    a_game.data.request_id = Uuid::new_v4().simple().to_string();
    a_game.data.action.name = "buy_spin".to_string();
    a_game.data.action.params.bet_per_line = Some(a_game.params.bet_per_line);
    a_game.data.action.params.lines = Some(a_game.params.line);
    a_game.data.action.params.bet_factor = Some(a_game.params.bet_factor);
    a_game.data.action.params.selected_mode = Some(a_game.params.selected_mode.to_string());
    a_game.request.params.clear();
    a_game.request.params.insert("gsc".to_string(), "play".to_string());
    a_game.request.body = serde_json::to_value(&Play::from(a_game.data.clone())).unwrap_or_default();
}

fn set_bonus_init(a_game: &mut Game) {
    a_game.data.command = "play".to_string();
    a_game.data.request_id = Uuid::new_v4().simple().to_string();
    a_game.data.action.name = "bonus_init".to_string();
    a_game.data.action.params.bet_per_line = None;
    a_game.data.action.params.lines = None;
    a_game.data.action.params.bet_factor = None;
    a_game.data.action.params.selected_mode = None;
    a_game.request.params.clear();
    a_game.request.params.insert("gsc".to_string(), "play".to_string());
    a_game.request.body = serde_json::to_value(&Play::from(a_game.data.clone())).unwrap_or_default();
}

fn set_respin(a_game: &mut Game) {
    a_game.data.command = "play".to_string();
    a_game.data.request_id = Uuid::new_v4().simple().to_string();
    a_game.data.action.name = "respin".to_string();
    a_game.data.action.params.bet_per_line = None;
    a_game.data.action.params.lines = None;
    a_game.data.action.params.bet_factor = None;
    a_game.data.action.params.selected_mode = None;
    a_game.request.params.clear();
    a_game.request.params.insert("gsc".to_string(), "play".to_string());
    a_game.request.body = serde_json::to_value(&Play::from(a_game.data.clone())).unwrap_or_default();
}

fn set_bonus_spins_stop(a_game: &mut Game) {
    a_game.data.command = "play".to_string();
    a_game.data.request_id = Uuid::new_v4().simple().to_string();
    a_game.data.action.name = "bonus_spins_stop".to_string();
    a_game.data.action.params.bet_per_line = None;
    a_game.data.action.params.lines = None;
    a_game.data.action.params.bet_factor = None;
    a_game.data.action.params.selected_mode = None;
    a_game.request.params.clear();
    a_game.request.params.insert("gsc".to_string(), "play".to_string());
    a_game.request.body = serde_json::to_value(&Play::from(a_game.data.clone())).unwrap_or_default();
}

fn next_body_exec(a_game: &mut Game, a_restart: &mut Restart) {
    if rand::random_range(0..1_000) == 0 {
        set_start(a_game);
    } else if let Some(context) = a_game.response.get("context") {
        if let Some(actions) = context.get("actions") {
            if actions.as_array().unwrap_or(&Vec::new()).contains(&Value::String("spin".to_string())) {
                if a_restart.bonus_spins_stop_befor && a_game.request.body.get("action").and_then(|action| action.get("name")).and_then(|name| name.as_str()) == Some("bonus_spins_stop") {
                    a_restart.bonus_spins_stop_befor = false; set_start(a_game);
                } else if a_restart.win && a_game.response.get("context").and_then(|context| context.get("spins")).and_then(|spins| spins.get("round_win")).and_then(|round_win| round_win.as_i64()) > Some(0) {
                    a_restart.win = false; set_start(a_game);
                } else if a_game.params.can_buy_bonus && a_game.params.buy_bonus_only {
                    set_buy_spin(a_game);
                } else if rand::random_range(0..100) == 0 {
                    set_sync(a_game);
                } else {set_spin(a_game);}
            } else if actions.as_array().unwrap_or(&Vec::new()).contains(&Value::String("bonus_init".to_string())) {
                if a_restart.buy_spin && a_game.request.body.get("action").and_then(|action| action.get("name")).and_then(|name| name.as_str()) == Some("buy_spin") {
                    a_restart.buy_spin = false; set_start(a_game);
                } else if a_restart.bonus_init {a_restart.bonus_init = false; set_start(a_game);} else {set_bonus_init(a_game);}
            } else if actions.as_array().unwrap_or(&Vec::new()).contains(&Value::String("respin".to_string())) {
                if a_restart.bonus_init_befor && a_game.request.body.get("action").and_then(|action| action.get("name")).and_then(|name| name.as_str()) == Some("bonus_init") {
                    a_restart.bonus_init_befor = false; set_start(a_game);
                } else if a_restart.respin && a_game.request.body.get("action").and_then(|action| action.get("name")).and_then(|name| name.as_str()) == Some("respin") {
                    a_restart.respin = false; set_start(a_game);
                } else {set_respin(a_game);}
            } else if actions.as_array().unwrap_or(&Vec::new()).contains(&Value::String("bonus_spins_stop".to_string())) {
                if a_restart.bonus_spins_stop {a_restart.bonus_spins_stop = false; set_start(a_game);} else {set_bonus_spins_stop(a_game);}
            }
        }
    }
}

