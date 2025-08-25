use serde_json::Value;
use std::io::{self, Write};
use std::fs;
use tokio;
use tokio::time::{sleep, Duration};
use rand;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
mod bng;
mod enj;
mod hacksaw;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("\x1B[2J\x1B[1;1H"); io::stdout().flush().unwrap();
    let supported_games: Vec<String> = serde_json::from_str(&(fs::read_to_string("./configs/supported_games.json".to_string()).unwrap_or_default())).unwrap_or_default();
    let supported_providers: Vec<String> = serde_json::from_str(&(fs::read_to_string("./configs/supported_providers.json".to_string()).unwrap_or_default())).unwrap_or_default();
    let mut rl = DefaultEditor::new()?;
    let history_path = "./configs/history.txt";
    let _ = rl.load_history(history_path);
    let game_provider = loop {
        match rl.readline("Input game provider (required): ") {
            Ok(line) => {
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() && supported_providers.contains(&trimmed) {
                    let _ = rl.add_history_entry(line.as_str());
                    if rl.append_history(history_path).is_err() {let _ = rl.save_history(history_path);}
                    break trimmed;
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => { return Ok(()); }
            Err(err) => { eprintln!("Error: {:?}", err); return Ok(()); }
        }
    };

    let game_name = loop {
        match rl.readline("Input game name (required): ") {
            Ok(line) => {
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() && supported_games.contains(&trimmed) {
                    let _ = rl.add_history_entry(line.as_str());
                    if rl.append_history(history_path).is_err() {let _ = rl.save_history(history_path);}
                    break trimmed;
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => { return Ok(()); }
            Err(err) => { eprintln!("Error: {:?}", err); return Ok(()); }
        }
    };
    loop {
        // config
        let config: Value = serde_json::from_str(&(fs::read_to_string("./configs/config.json").unwrap_or_default())).unwrap_or_default();
        let must_delay_between_requests = config.get("must_delay_between_requests").and_then(|v| v.as_bool()).unwrap_or(true);
        let delay_between_requests = config.get("delay_between_requests").and_then(|v| {if v.as_i64() < Some(1000) && v.as_i64() != Some(0) {Some(1000)} else {v.as_i64()}}).unwrap_or(1000);
        let location = config.get("location").and_then(|v| v.as_str()).unwrap_or("./");
        
        let _ = match game_provider.as_str() {
            "bng" => {bng::execute(game_name.clone(), location.to_string(), must_delay_between_requests, delay_between_requests).await}
            "enj" => {enj::execute(game_name.clone(), location.to_string(), must_delay_between_requests, delay_between_requests).await}
            "hacksaw" => {hacksaw::execute(game_name.clone(), location.to_string(), must_delay_between_requests, delay_between_requests).await}
            _ => {eprintln!("\r\tProvider not implement"); Ok(())}
        };
        let delay: u64 = rand::random_range(10..=30);
        print!("\x1B[K\t\n");
        for sec in (0..delay).rev() {
            print!("\x1B[1A\x1B[2K");
            let _ = io::stdout().flush();
            print!("\x1B[KStart new game: {}\n", sec);
            sleep(Duration::from_millis(1_000)).await;
        }
        print!("\x1B[1A\x1B[2K");
    }
}