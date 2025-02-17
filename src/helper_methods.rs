use crate::errors::AppError;
use reqwest::{RequestBuilder, Response};
use std::time::Duration;

/// Sends a GET request to the desired URL, retrying with the desired amount of times if it fails.
///
/// Every failed attempt will wait for the passed in time.
///
/// # Errors
/// - Failed to get a response after the desired amount of attempts.
/// - Could not clone the request.
pub async fn get_with_retry(
  request: RequestBuilder,
  retry_count: usize,
  wait_time: Duration,
) -> Result<Response, AppError> {
  let request_string = format!("{:?}", request);

  for iteration in 1..=retry_count {
    let Some(request) = request.try_clone() else {
      return Err(AppError::RequestCouldNotBeCloned(request_string));
    };
    let result = request.send().await;

    if let Ok(response) = result {
      return Ok(response);
    } else {
      tracing::warn!(
        "Failed to get a response from {:?}. {} more attempts left",
        request_string,
        retry_count - iteration
      );
      tokio::time::sleep(wait_time).await;

      continue;
    }
  }

  Err(AppError::RanOutOfGetRequestAttempts {
    request: format!("{:?}", request),
    attempts: retry_count,
  })
}
