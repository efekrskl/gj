use crate::database::Log;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use log::debug;
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::ListItem;
use std::collections::BTreeMap;
use std::process::Command;
use time::{Date, PrimitiveDateTime, format_description};

pub fn to_iso8601_timestamp(input: &str) -> Result<String> {
    let date = NaiveDate::parse_from_str(input.trim(), "%d.%m.%Y")
        .context("Invalid date format. Please use DD.MM.YYYY.")?;
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();

    Ok(datetime.format("%Y-%m-%dT%H:%M:%S").to_string())
}

pub fn get_git_activity(email: &str, since: Option<String>) -> Result<String> {
    if email.is_empty() {
        return Ok("Could not detect git user.email".to_string());
    }
    let since = since.unwrap_or("midnight".to_string());

    debug!("Running git log since: {since} email: {email}");

    let log_output = Command::new("git")
        .args([
            "log",
            &format!("--author={}", email),
            &format!("--since={}", since),
            "--all",
            "--no-merges",
            "--pretty=format:- %s",
        ])
        .output()
        .context("Failed to run git log")?;

    debug!("Git log output: {:?}", log_output);

    let logs = String::from_utf8_lossy(&log_output.stdout).to_string();

    if logs.is_empty() {
        Ok("No git activity found today.".to_string())
    } else {
        Ok(logs)
    }
}

pub fn logs_to_list_items(logs: &[Log]) -> Vec<ListItem> {
    let fmt = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

    let logs = logs
        .iter()
        .map(|log| {
            let date_time = PrimitiveDateTime::parse(&log.created_at, &fmt)
                .map(|dt| dt.time())
                .ok();
            let time_str = date_time
                .map(|t| format!("{:02}:{:02}", t.hour(), t.minute()))
                .unwrap_or_else(|| "??:??".to_string());

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", time_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(&log.content),
            ]))
        })
        .collect();

    logs
}

pub fn logs_by_day_map(logs: Vec<Log>) -> Result<BTreeMap<Date, Vec<Log>>> {
    let mut map: BTreeMap<Date, Vec<Log>> = BTreeMap::new();

    for log in logs {
        let fmt = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")?;
        let date = PrimitiveDateTime::parse(&log.created_at, &fmt)?.date();

        map.entry(date).or_default().push(log);
    }

    Ok(map)
}
