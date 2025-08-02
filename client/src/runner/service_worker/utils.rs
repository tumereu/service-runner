/// Turns a `reqwest::Error` into a clean, human-readable string.
pub fn format_reqwest_error(err: &reqwest::Error) -> String {
    if err.is_connect() {
        return format!("Connection error: {}", err);
    }
    if err.is_timeout() {
        return "Request timed out.".to_string();
    }
    if err.is_request() {
        return format!("Request build error: {}", err);
    }
    if err.is_body() {
        return format!("Body error: {}", err);
    }
    if err.is_decode() {
        return format!("Response decoding error: {}", err);
    }

    // Fall back to a generic error message
    format!("Unexpected error: {}", err)
}