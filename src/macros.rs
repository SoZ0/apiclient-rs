#[macro_export]
macro_rules! create_api_submodule {
    ($client_name:ident, $base_path:expr) => {
        #[derive(Debug, Clone)]
        pub struct $client_name {
            client: std::sync::Arc<apiclient_rs::ApiClient>,
            base_path: String,
        }

        impl $client_name {
            pub fn new(client: std::sync::Arc<apiclient_rs::ApiClient>) -> Self {
                Self {
                    client: client,
                    base_path: $base_path.to_string(),
                }
            }

            pub async fn get<T, B>(&self, endpoint: &str, params: Option<&B>) -> apiclient_rs::ApiResult<T>
            where
                T: serde::de::DeserializeOwned,
                B: serde::Serialize,
            {
                let full_endpoint = format!("{}{}", self.base_path, endpoint);
                let client = self.client.as_ref();
                let query_params = self.client.as_ref().serialize_params(params)?;
                let response = client.get(&full_endpoint, query_params.as_deref()).await?;
                self.client.as_ref().deserialize_response(response)
            }

            pub async fn post<T, B>(&self, endpoint: &str, body: Option<&B>) -> apiclient_rs::ApiResult<T>
            where
                T: serde::de::DeserializeOwned,
                B: serde::Serialize,
            {
                let full_endpoint = format!("{}{}", self.base_path, endpoint);
                self.client
                    .as_ref()
                    .post(&full_endpoint, body)
                    .await
            }
        }
    };
}

#[macro_export]
macro_rules! define_api_endpoint {
    // When parameters are provided
    (
        $(#[$meta:meta])*
        impl $impl_target:ty;
        fn $fn_name:ident(
            &self $(, $path_param:ident : $path_type:ty)* $(,)?
            $(; required_params: {$($req_param:ident : $req_type:ty),* $(,)?})?
            $(; optional_params: {$($opt_param:ident : $opt_type:ty),* $(,)?})?
        ) -> $response_type:ident;
        method: $method:ident;
        endpoint: $endpoint_fmt:expr;
        response_fields: {
            $(
                $(#[$field_meta:meta])*
                $resp_field:ident : $resp_type:ty
            ),* $(,)?
        }
    ) => {
        paste::paste! {
            // Define the parameter struct
            #[derive(Debug, Clone, serde::Serialize, derive_builder::Builder)]
            #[builder(public)]
            #[serde(rename_all = "camelCase")]
            pub struct [<$fn_name:camel Params>] {
                $(
                    $(
                        #[builder(setter(into))]
                        pub $req_param: $req_type,
                    )*
                )?
                $(
                    $(
                        #[builder(setter(into), default)]
                        pub $opt_param: Option<$opt_type>,
                    )*
                )?
            }

            // Define the response struct
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct $response_type {
                $(
                    $(#[$field_meta])*
                    pub $resp_field: $resp_type,
                )*
            }

            impl $impl_target {
                $(#[$meta])*
                pub async fn $fn_name(
                    &self,
                    $(
                        $path_param : $path_type,
                    )*
                    params: &[<$fn_name:camel Params>],
                ) -> apiclient_rs::ApiResult<$response_type> {
                    let endpoint = format!($endpoint_fmt, $($path_param),*);
                    self.$method(&endpoint, Some(&params)).await
                }
            }
        }
    };

    // When no parameters are provided
    (
        $(#[$meta:meta])*
        impl $impl_target:ty;
        fn $fn_name:ident(
            &self $(, $path_param:ident : $path_type:ty)* $(,)?
        ) -> $response_type:ident;
        method: $method:ident;
        endpoint: $endpoint_fmt:expr;
        response_fields: {
            $(
                $(#[$field_meta:meta])*
                $resp_field:ident : $resp_type:ty
            ),* $(,)?
        }
    ) => {
        paste::paste! {
            // Define the response struct
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct $response_type {
                $(
                    $(#[$field_meta])*
                    pub $resp_field: $resp_type,
                )*
            }

            impl $impl_target {
                $(#[$meta])*
                pub async fn $fn_name(
                    &self,
                    $(
                        $path_param : $path_type,
                    )*
                ) -> apiclient_rs::ApiResult<$response_type> {
                    let endpoint = format!($endpoint_fmt, $($path_param),*);
                    self.$method(&endpoint, None::<&()>).await
                }
            }
        }
    };
}


#[macro_export]
macro_rules! create_api_schema {
    // When parameters are provided
    (
        $(#[$meta:meta])*
        $schema:ident;
        values: {
            $(
                $(#[$field_meta:meta])*
                $resp_field:ident : $resp_type:ty
            ),* $(,)?
        }
    ) => {
        // Define the response struct
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct $schema {
            $(
                $(#[$field_meta])*
                pub $resp_field: $resp_type,
            )*
        }
    };
}
