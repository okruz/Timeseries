mod json_handler;
use std::collections::HashMap;

struct Echo {}

impl json_handler::FunctionHandler for Echo {
    fn handle(
        &self,
        json: json::JsonValue,
    ) -> Result<json::JsonValue, json_handler::HandlingError> {
        Ok(json)
    }
}

struct Add {}

impl json_handler::FunctionHandler for Add {
    fn handle(
        &self,
        mut json: json::JsonValue,
    ) -> Result<json::JsonValue, json_handler::HandlingError> {
        if !json["lol"].is_null() {
            json["lol"] = "fluebls".into();
            json["nonlol"] = "mehrso".into();
            Ok(json)
        } else {
            Err(json_handler::HandlingError {
                message: "Kein lol".to_string(),
                code: 14,
            })
        }
    }
}

type Handler = Box<dyn json_handler::FunctionHandler>;

fn main() {
    let handlers: HashMap<&str, Handler> = HashMap::from([
        ("Echo", Box::new(Echo {}) as Handler),
        ("Add", Box::new(Add {}) as Handler),
    ]);

    let msg_json = json::parse("{\"lol\": \"hgnla\"}");

    if let Ok(msg_json) = msg_json {
        if let Ok(response) = handlers["Echo"].handle(msg_json.clone()) {
            println!("Echo: {}", json::stringify(response));
        }
        if let Ok(response) = handlers["Add"].handle(msg_json.clone()) {
            println!("Add: {}", json::stringify(response));
        } else {
            println!("Error");
        }
    }
}
