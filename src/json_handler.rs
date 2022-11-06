use crate::errors::HandlingError;
use serde_json::Value;

pub trait FunctionHandler {
    fn handle(&self, json: Value) -> Result<Value, HandlingError>;
}
