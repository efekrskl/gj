use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::process::Command;

pub fn to_iso8601_timestamp(input: &str) -> Result<String> {
    let date = NaiveDate::parse_from_str(input.trim(), "%d.%m.%Y")
        .context("Invalid date format. Please use DD.MM.YYYY.")?;
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();

    Ok(datetime.format("%Y-%m-%dT%H:%M:%S").to_string())
}

pub fn get_git_activity(email: &str) -> Result<String> {
    if email.is_empty() {
        return Ok("Could not detect git user.email".to_string());
    }

    let log_output = Command::new("git")
        .args([
            "log",
            &format!("--author={}", email),
            "--since=midnight",
            "--all",
            "--no-merges",
            "--pretty=format:- %s",
        ])
        .output()
        .context("Failed to run git log")?;

    let logs = String::from_utf8_lossy(&log_output.stdout).to_string();

    if logs.is_empty() {
        Ok("No git activity found today.".to_string())
    } else {
        Ok(logs)
    }
}
