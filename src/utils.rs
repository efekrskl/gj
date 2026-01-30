use anyhow::{Context, Result};
use chrono::NaiveDate;

pub fn to_iso8601_timestamp(input: &str) -> Result<String> {
    let date = NaiveDate::parse_from_str(input.trim(), "%d.%m.%Y")
        .context("Invalid date format. Please use DD.MM.YYYY.")?;
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();

    Ok(datetime.format("%Y-%m-%dT%H:%M:%S").to_string())
}
