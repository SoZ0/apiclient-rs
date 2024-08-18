
use reqwest::RequestBuilder;use std::fmt::{self, Debug};
pub trait AuthStrategy: Send + Sync {
    fn apply_auth(&self, request: RequestBuilder) -> RequestBuilder;
}

impl Debug for dyn AuthStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AuthStrategy")
    }
}

pub struct ApiKeyAuth {
    api_key: String,
}

impl ApiKeyAuth {
    pub fn new(api_key: String) -> Self {
        ApiKeyAuth { api_key }
    }
}

impl AuthStrategy for ApiKeyAuth {
    fn apply_auth(&self, request: RequestBuilder) -> RequestBuilder {
        request.header("x-api-key", &self.api_key)
    }
}

impl Debug for ApiKeyAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiKeyAuth")
            .field("api_key", &"***") // Don't expose the actual key
            .finish()
    }
}

pub struct BearerAuth {
    token: String,
}

impl BearerAuth {
    pub fn new(token: String) -> Self {
        BearerAuth { token }
    }
}

impl AuthStrategy for BearerAuth {
    fn apply_auth(&self, request: RequestBuilder) -> RequestBuilder {
        request.bearer_auth(&self.token)
    }
}

impl Debug for BearerAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BearerAuth")
            .field("token", &"***") // Don't expose the actual token
            .finish()
    }
}