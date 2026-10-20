#[derive(Debug, Clone, PartialEq)]
pub struct RequestSavedModel {
    pub id: i64,

    pub title: String,
    pub url: String,
    pub method: String,

    pub request_body: Option<String>,
    pub request_headers: Option<String>,
    pub request_params: Option<String>,

    pub response_body: Option<String>,
    pub response_headers: Option<String>,
    pub response_cookies: Option<String>,
}
