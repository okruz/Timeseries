use serde_json::Value;

pub struct HandlingError {
    pub message: String,
    pub code: i32,
}

pub trait FunctionHandler {
    fn handle(&self, json: Value) -> Result<Value, HandlingError>;
}
