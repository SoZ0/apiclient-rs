
pub trait QueryParams {
    fn to_query_params(&self) -> Vec<(String, String)>;
}