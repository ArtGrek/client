use serde_json::Value;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub fn log_request_response<P>(transactions_file_name: P, content: &Value) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(transactions_file_name)?;
    writeln!(file, "{},", content.to_string())
}