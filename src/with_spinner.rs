use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub async fn with_spinner<F, T>(
    function: F,
    loading_message: String,
    success_message: Option<String>,
    failure_message: Option<String>,
) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner} {msg}")?,
    );
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message(loading_message);

    let result = function.await;

    match &result {
        Ok(_) => {
            if let Some(success_message) = success_message {
                spinner.finish_with_message(success_message);
            } else {
                spinner.finish_and_clear();
            }
        }
        Err(_) => {
            if let Some(failure_message) = failure_message {
                spinner.finish_with_message(failure_message);
            } else {
                spinner.finish_and_clear();
            }
        }
    }

    result
}
