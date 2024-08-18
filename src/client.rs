use reqwest::{Client as ReqwestClient, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::error::ApiClientError;
use crate::auth::AuthStrategy;
use tracing::{info, debug, error, instrument};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub type    ApiResult<T> = Result<T, ApiClientError>;

#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
    client: ReqwestClient,
    auth_strategy: Option<Arc<dyn AuthStrategy>>, // Using Arc to allow cloning
}

impl ApiClient {
    pub fn new(base_url: &str, auth_strategy: Option<Arc<dyn AuthStrategy>>) -> Self {
        ApiClient {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: ReqwestClient::new(),
            auth_strategy,
        }
    }

    fn apply_auth(&self, request: RequestBuilder) -> RequestBuilder {
        if let Some(auth) = &self.auth_strategy {
            auth.apply_auth(request)
        } else {
            request
        }
    }

    #[instrument(skip(self))]
    pub async fn get<T>(&self, endpoint: &str, params: Option<&[(&str, &str)]>) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/{}", self.base_url, endpoint.trim_start_matches('/'));
        info!("Sending GET request to URL: {}", url);

        let mut request = self.client.get(&url);
        request = self.apply_auth(request);

        if let Some(params) = params {
            request = request.query(params);
            debug!("Added query parameters: {:?}", params);
        }

        self.execute_request(request).await
    }

    #[instrument(skip(self, body))]
    pub async fn post<T, B>(&self, endpoint: &str, body: Option<&B>) -> ApiResult<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = format!("{}/{}", self.base_url, endpoint.trim_start_matches('/'));
        info!("Sending POST request to URL: {}", url);

        let mut request = self.client.post(&url);
        request = self.apply_auth(request);

        if let Some(body) = body {
            request = request.json(body);

            match serde_json::to_string(body) {
                Ok(json_body) => {
                    debug!("Serialized body: {}", json_body);
                }
                Err(e) => {
                    error!("Failed to serialize body: {:?}", e);
                    return Err(ApiClientError::DeserializeError(e.to_string()));
                }
            }
        }

        debug!("Sending request {:?}", request);

        let response = request.send().await.map_err(|e| {
            error!("Network error while sending POST request to {}: {:?}", url, e);
            ApiClientError::Network(e)
        })?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    async fn execute_request<T>(&self, request: RequestBuilder) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        let mut retries = 3;

        while retries > 0 {
            let response = request.try_clone().unwrap().send().await.map_err(|e| {
                error!("Network error while sending request: {:?}", e);
                ApiClientError::Network(e)
            })?;

            match self.handle_response(response).await {
                Ok(result) => return Ok(result),
                Err(ApiClientError::RateLimit(ref message)) => {
                    error!("Rate limit exceeded: {}", message);
                    retries -= 1;
                    sleep(Duration::from_secs(2)).await;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Err(ApiClientError::MaxRetriesReached)
    }

    #[instrument(skip(self))]
    async fn handle_response<T>(&self, response: Response) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| "Failed to read response body".to_string());

        if status.is_success() {
            // First, try to deserialize the response
            serde_json::from_str::<T>(&body).map_err(|err| {
                error!("Failed to parse JSON response: {:?}", err);
                ApiClientError::JsonParse(err)
            })
        } else if status == StatusCode::TOO_MANY_REQUESTS {
            Err(ApiClientError::RateLimit(body))
        } else {
            Err(ApiClientError::ApiError { status, body })
        }
    }
}
