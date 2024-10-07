
use reqwest::RequestBuilder;use std::fmt::{self, Debug};
pub trait AuthStrategy: Send + Sync {
    fn apply_auth(&self, request: RequestBuilder) -> RequestBuilder;
}

impl Debug for dyn AuthStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AuthStrategy")
    }
}

pub struct HeaderAuth {
    header: String,
    api_key: String,
}

impl HeaderAuth {
    pub fn new(header:String, api_key: String) -> Self {
        HeaderAuth { header, api_key }
    }
}

impl AuthStrategy for HeaderAuth {
    fn apply_auth(&self, request: RequestBuilder) -> RequestBuilder {
        request.header(&self.header, &self.api_key)
    }
}

impl Debug for HeaderAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeaderAuth")
            .field(&self.header, &"***") // Don't expose the actual key
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