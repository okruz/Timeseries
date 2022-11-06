use crate::dao::Dao;
use crate::errors::HandlingError;
use futures_channel::mpsc::UnboundedSender;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;

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

type Handler = Arc<dyn FunctionHandler>;

fn get_handler_map(dao: Arc<Dao>) -> HashMap<String, Handler> {
    HashMap::from([(
        "GetAllPlots".to_string(),
        Arc::new(GetAllPlots { dao: dao.clone() }) as Handler,
    )])
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

    pub fn dispatch(&self, message: &Message, tx: &Tx) {
        if let Ok(msg_string) = message.to_text() {
            println!("Received: \"{}\".", msg_string);
            if let Ok(json_value) = serde_json::from_str(msg_string) {
                self.dispatch_internal(json_value, tx);
            } else {
                println!("Could not parse JSON.");
            }
        }
    }

    fn dispatch_internal(&self, mut json_rpc: Value, tx: &Tx) {
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

                let message = Message::text(response);
                match tx.unbounded_send(message) {
                    Ok(_) => println!("Send successful"),
                    Err(_) => println!("Send failed"),
                };
            }
        }
    }
}
