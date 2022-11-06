//! A chat server that broadcasts a message to all connections.
//!
//! This is a simple line-based server which accepts WebSocket connections,
//! reads lines from those connections, and broadcasts the lines to all other
//! connected clients.
//!
//! You can test this out by running:
//!
//!     cargo run --example server 127.0.0.1:12345
//!
//! And then in another window run:
//!
//!     cargo run --example client ws://127.0.0.1:12345/
//!
//! You can run the second command in multiple windows and then chat between the
//! two, seeing the messages from the other client as they're received. For all
//! connected clients they'll all join the same room and see everyone else's
//! messages.
mod dao;
mod data_model;
mod errors;
mod json_handler;

use std::{env, io::Error as IoError, io::ErrorKind, net::SocketAddr, sync::Arc};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

use dao::Dao;
use data_model::Plot;
use json_handler::Dispatcher;

use nix;

type Tx = UnboundedSender<Message>;
type Dp = Arc<Dispatcher>;

pub fn privdrop(user: &str, group: &str) -> Result<(), nix::Error> {
    match nix::unistd::Group::from_name(group)? {
        Some(group) => nix::unistd::setgid(group.gid),

        None => Err(nix::Error::last()),
    }?;

    match nix::unistd::User::from_name(user)? {
        Some(user) => nix::unistd::setuid(user.uid),

        None => Err(nix::Error::last()),
    }
}

async fn handle_connection(dispatcher: Dp, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (tx, rx) = unbounded();

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        tx.unbounded_send(Message::text("Got your message!".to_string()));
        dispatcher.dispatch(&msg, &tx);
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let mut dao =
        Dao::new_in_memory().or(Err(IoError::new(ErrorKind::Other, "Database error.")))?;
    dao.add_plot(&Plot {
        id: 0,
        name: "entry".to_string(),
        description: "desc".to_string(),
        time_series: vec![],
    })
    .or(Err(IoError::new(ErrorKind::Other, "Database error.")))?;
    let dispatcher = Arc::new(Dispatcher::new(dao));
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let user = env::args().nth(2);
    let group = env::args().nth(3);

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    if user != None && group != None {
        privdrop(&user.unwrap(), &group.unwrap()).expect("Privilege drop failed.");
    } else {
        println!("No user/group privileges to drop to specified.");
    }

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        handle_connection(dispatcher.clone(), stream, addr).await;
    }

    Ok(())
}
