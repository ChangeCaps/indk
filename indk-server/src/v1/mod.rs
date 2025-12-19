use std::sync::Arc;

use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::Message},
    response::IntoResponse,
    routing::any,
};
use futures::{SinkExt, StreamExt};
use indk_proto::v1::{Item, Request, Response};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::ServerState;

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub items: Vec<Item>,
}

pub async fn router() -> Router<Arc<ServerState>> {
    Router::new().route("/ws", any(ws))
}

async fn ws(State(server): State<Arc<ServerState>>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(async move |socket| {
        let (mut sender, receiver) = socket.split();

        let mut receiver = receiver.map(|message| -> eyre::Result<Request> {
            let message = message?;
            let text = message.to_text()?;
            Ok(serde_json::from_str(text)?)
        });

        let (update_sender, mut update_receiver) = unbounded_channel();
        server.senders.write().push(update_sender.clone());

        loop {
            tokio::select! {
                Some(response) = update_receiver.recv() => {
                    if let Ok(json) = serde_json::to_string(&response) {
                        let _ = sender.send(Message::text(json)).await;
                    }
                }

                request = receiver.next() => {
                    let Some(Ok(request)) = request else {
                        break;
                    };

                    let response = respond(&server, &update_sender, request).await;

                    if let Ok(json) = serde_json::to_string(&response) {
                        let _ = sender.send(Message::text(json)).await;
                    }
                }
            }
        }
    })
}

async fn respond(server: &ServerState, sender: &UnboundedSender<Response>, request: Request) {
    match request {
        Request::GetItems => {
            let _ = sender.send(Response::Items(server.get_items()));
        }

        Request::CreateItem(item) => {
            let index = {
                let mut order = server.items.order.write();
                order.push(item.id);
                order.len() - 1
            };

            server.items.items.insert(item.id, item.clone());
            server.send_all(Some(sender), Response::ItemCreated { item, index });
        }

        Request::RemoveItem(id) => {
            let _ = server.items.items.remove(&id);

            let mut order = server.items.order.write();
            if let Some(index) = order.iter().position(|x| *x == id) {
                order.remove(index);
                server.send_all(Some(sender), Response::ItemRemoved { id, index });
            }
        }

        Request::RenameItem { id, name } => {
            if let Some(mut item) = server.items.items.get_mut(&id) {
                item.name = name.clone();
            }

            server.send_all(Some(sender), Response::ItemRenamed { id, name });
        }

        Request::CompleteItem { id, completed } => {
            if let Some(mut item) = server.items.items.get_mut(&id) {
                item.completed = completed;
            }

            server.send_all(Some(sender), Response::ItemCompleted { id, completed });
        }
    }
}
