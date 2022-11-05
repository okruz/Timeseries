mod data_model;
mod json_handler;
use data_model::TimeSeries;
use json_handler::{FunctionHandler, HandlingError};
use serde_json::{Error, Value};
use std::collections::HashMap;

type GenericJson = Result<Value, Error>;

struct Echo {}

impl FunctionHandler for Echo {
    fn handle(&self, json: Value) -> Result<Value, HandlingError> {
        Ok(json)
    }
}

struct Add {}

impl FunctionHandler for Add {
    fn handle(&self, mut json: Value) -> Result<Value, HandlingError> {
        if !json["lol"].is_null() {
            json["lol"] = "fluebls".into();
            json["nonlol"] = "mehrso".into();
            Ok(json)
        } else {
            Err(HandlingError {
                message: "Kein lol".to_string(),
                code: 14,
            })
        }
    }
}

type Handler = Box<dyn FunctionHandler>;

fn get_handler_map() -> HashMap<&'static str, Handler> {
    HashMap::from([
        ("Echo", Box::new(Echo {}) as Handler),
        ("Add", Box::new(Add {}) as Handler),
    ])
}

fn main() {
    let handlers = get_handler_map();

    let msg_json: GenericJson = serde_json::from_str("{\"lol\": \"hgnla\"}");

    if let Ok(msg_json) = msg_json {
        if let Ok(response) = handlers["Echo"].handle(msg_json.clone()) {
            println!("Echo: {}", response);
        }

        if let Ok(response) = handlers["Add"].handle(msg_json.clone()) {
            println!("Add: {}", response);
        } else {
            println!("Error");
        }
    }

    let time_series = TimeSeries {
        id: 1,
        name: "It's the name".to_string(),
        unit: "kg".to_string(),
        time_points: vec![],
        values: vec![],
    };

    let time_series_json: Value = (&time_series).into();
    println!("TimeSeries: {}", time_series_json);
    let time_series2: TimeSeries = (&time_series_json).try_into().unwrap();
    let time_series_json2: Value = (&time_series2).into();
    println!("TimeSeries2: {}", time_series_json2);
}
