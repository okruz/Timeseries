mod dao;
mod data_model;
mod errors;
mod json_handler;
use chrono::Utc;
use data_model::{Plot, TimeSeries, TimeSeriesEntry};
use errors::HandlingError;
use json_handler::FunctionHandler;
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
        time_points: vec![Utc::now()],
        values: vec![44.0],
    };

    let time_series_json: Value = (&time_series).into();
    println!("TimeSeries: {}", time_series_json);
    let time_series2: TimeSeries = (&time_series_json).try_into().unwrap();
    let time_series_json2: Value = (&time_series2).into();
    println!("TimeSeries2: {}", time_series_json2);

    let plot = Plot {
        id: 0,
        name: "cool".to_string(),
        description: "cool data".to_string(),
        time_series: vec![
            TimeSeries {
                id: 0,
                name: "cool heat value".to_string(),
                unit: "a few K".to_string(),
                time_points: vec![Utc::now().into()],
                values: vec![3.1],
            },
            TimeSeries {
                id: 0,
                name: "a bit hotter heat value".to_string(),
                unit: "a few nore K".to_string(),
                time_points: vec![Utc::now().into()],
                values: vec![63.898],
            },
        ],
    };

    if let Ok(mut dao) = dao::Dao::new_in_memory() {
        match dao.add_plot(&plot) {
            Ok(plot) => println!("Successfully added {:?}.", plot),
            Err(err) => println!("Error: {:?}", err),
        };

        match dao.add_time_series(1, &time_series2) {
            Ok(time_series) => println!("Successfully added {:?}.", time_series),
            Err(err) => println!("Error: {:?}", err),
        };

        let entry = TimeSeriesEntry {
            time_point: Utc::now(),
            value: 48588.3,
        };

        match dao.add_entry(2, &entry) {
            Ok(_) => println!("Added entry."),
            Err(err) => println!("Error: {:?}", err),
        };

        match dao.get_time_series(1, true, None) {
            Ok(time_series) => println!("{:?}", time_series),
            Err(err) => println!("Error: {:?}", err),
        };

        match dao.get_time_series(2, true, None) {
            Ok(time_series) => println!("{:?}", time_series),
            Err(err) => println!("Error: {:?}", err),
        };

        match dao.get_time_series(3, true, None) {
            Ok(time_series) => println!("{:?}", time_series),
            Err(err) => println!("Error: {:?}", err),
        };

        match dao.get_plot(1, false, None) {
            Ok(plot) => println!("{:?}.", plot),
            Err(err) => println!("Error: {:?}", err),
        };

        match dao.get_plot(1, true, None) {
            Ok(plot) => println!("{:?}.", plot),
            Err(err) => println!("Error: {:?}", err),
        };
    }
}
