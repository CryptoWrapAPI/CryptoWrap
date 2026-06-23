use axum::http::StatusCode;
use reqwest::Client;
use serde::Serialize;

/// Notify a shop about a payment status change.
/// Sends a POST request with the payment data to the notify_url.
/// Retries up to 3 times, expects HTTP 202 to consider it successful.
pub async fn notify_shop<T: Serialize>(
    notify_url: &str,
    payload: &T,
) -> Result<(), String> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let max_retries = 3;

    for attempt in 1..=max_retries {
        let response = client
            .post(notify_url)
            .timeout(tokio::time::Duration::from_secs(5))
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status() == StatusCode::ACCEPTED {
            return Ok(());
        }

        if attempt < max_retries {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    Err(format!(
        "Failed to notify shop after {} attempts",
        max_retries
    ))
}
