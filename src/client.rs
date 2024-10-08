use reqwest::{Client as ReqwestClient, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
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
    pub async fn get<T>(&self, endpoint: &str, params: Option<&[(String, String)]>) -> ApiResult<T>
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

    pub fn serialize_params<B>(&self, params: Option<&B>) -> ApiResult<Option<Vec<(String, String)>>>
    where
        B: Serialize,
    {

        fn convert_json_to_pairs(json_string: &str) -> Result<Vec<(String, String)>, ApiClientError> {
            let value: Value = serde_json::from_str(json_string)?;
            let mut pairs: Vec<(String, String)> = Vec::new();
        
            if let Value::Object(map) = value {
                for (key, value) in map.iter() {
                    let key_str = key.clone();
                    let value_str = match value {
                        Value::String(s) => s.clone(),
                        Value::Bool(b) => b.to_string(),
                        Value::Number(n) => n.to_string(),
                        _ => continue,
                    };
                    pairs.push((key_str, value_str));
                }
            }
        
            Ok(pairs)
        }

        if let Some(params) = params {
            let json_string = serde_json::to_string(params)?;
            let pairs = convert_json_to_pairs(&json_string)?;
            Ok(Some(pairs))
        } else {
            Ok(None)
        }
    }

    pub fn deserialize_response<T>(&self, response: Value) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        let result = serde_path_to_error::deserialize::<_, T>(response);
        result.map_err(|err| {
            let path = err.path().to_string();
            error!("Deserialization error at {}: {}", path, err);
            ApiClientError::DeserializeError(err.to_string())
        })
    }
}
