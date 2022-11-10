use crate::dao::Dao;
use crate::data_model::time_point_from_str;
use crate::errors::HandlingError;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tungstenite::protocol::Message;

pub trait FunctionHandler {
    fn handle(&self, json: Value) -> Result<Value, HandlingError>;
}

struct GetAllPlots {
    dao: Arc<Dao>,
}

impl FunctionHandler for GetAllPlots {
    fn handle(&self, _json: Value) -> Result<Value, HandlingError> {
        let plots = self.dao.get_all_plots()?;
        let mut plots_json: Vec<Value> = vec![];
        for plot in plots {
            plots_json.push((&plot).into());
        }

        Ok(plots_json.into())
    }
}

struct GetPlot {
    dao: Arc<Dao>,
}

impl FunctionHandler for GetPlot {
    fn handle(&self, json: Value) -> Result<Value, HandlingError> {
        if let Some(id) = json["Id"].as_i64() {
            let start_date = match json["StartDate"]
                .as_str()
                .map(|date_string| time_point_from_str(&date_string))
            {
                None => Ok(None),
                Some(Ok(date_time)) => Ok(Some(date_time)),
                Some(Err(err)) => Err(err),
            }?;

            let plot = self.dao.get_plot_with_data(id, start_date)?;
            return Ok((&plot).into());
        }

        Err(HandlingError {
            message: "Id missing".to_string(),
            code: 420,
        })
    }
}

type Handler = Arc<dyn FunctionHandler>;

fn get_handler_map(dao: Arc<Dao>) -> HashMap<String, Handler> {
    HashMap::from([
        (
            "GetAllPlots".to_string(),
            Arc::new(GetAllPlots { dao: dao.clone() }) as Handler,
        ),
        (
            "GetPlot".to_string(),
            Arc::new(GetPlot { dao: dao.clone() }) as Handler,
        ),
    ])
}

pub struct Dispatcher {
    handler: HashMap<String, Handler>,
}

impl Dispatcher {
    pub fn new(dao: Dao) -> Self {
        Self {
            handler: get_handler_map(Arc::new(dao)),
        }
    }

    pub fn dispatch(&self, message: &Message) -> Option<Message> {
        if let Ok(msg_string) = message.to_text() {
            if let Ok(json_value) = serde_json::from_str(msg_string) {
                return self.dispatch_internal(json_value);
            } else {
                println!("Could not parse JSON.");
            }
        }
        None
    }

    fn dispatch_internal(&self, mut json_rpc: Value) -> Option<Message> {
        let id = json_rpc["id"].take();

        if !id.is_null() {
            if let Some(method) = json_rpc["method"].take().as_str() {
                let response = self.handler[method].handle(json_rpc["params"].take());

                let response = match response {
                    Ok(result) => json![{"jsonrpc": "2.0", "id": id, "result": result}],
                    Err(err) => {
                        json![{"jsonrpc": "2.0", "id": id, "error": {"message": err.message, "code": err.code}}]
                    }
                }.to_string();
                return Some(Message::text(response));
            }
        }
        None
    }
}
