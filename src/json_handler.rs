use json;

pub struct HandlingError {
    pub message: String,
    pub code: i32,
}

pub trait FunctionHandler {
    fn handle(&self, json: json::JsonValue) -> Result<json::JsonValue, HandlingError>;
}
