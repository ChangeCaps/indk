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
use uuid::Uuid;

use crate::ServerState;

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub lists: Vec<List>,
    pub items: Vec<Item>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct List {
    pub id: Uuid,
    pub name: String,
    pub items: Vec<Uuid>,
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

                    respond(&server, &update_sender, request).await;
                }
            }
        }
    })
}

async fn respond(server: &ServerState, sender: &UnboundedSender<Response>, request: Request) {
    match request {
        Request::GetLists {} => {
            let lists = server
                .lists
                .lists
                .iter()
                .map(|list| indk_proto::v1::List {
                    id: list.id,
                    name: list.name.clone(),
                })
                .collect();

            let _ = sender.send(Response::Lists { lists });
        }

        Request::CreateList { list } => {
            server.lists.lists.insert(
                list.id,
                List {
                    id: list.id,
                    name: list.name.clone(),
                    items: Vec::new(),
                },
            );

            let index = {
                let mut order = server.lists.order.write();
                order.push(list.id);
                order.len() - 1
            };

            server.send_all(Some(sender), Response::ListCreated { list, index });
        }

        Request::RemoveList { list } => {
            let _ = server.lists.lists.remove(&list);

            server.send_all(Some(sender), Response::ListRemoved { list });
        }

        Request::RenameList { list, name } => {
            if let Some(mut list) = server.lists.lists.get_mut(&list) {
                list.name = name.clone();
            }

            server.send_all(Some(sender), Response::ListRenamed { list, name });
        }

        Request::GetItems { list } => {
            let Some(list) = server.lists.lists.get(&list) else {
                return;
            };

            let items = list
                .items
                .iter()
                .filter_map(|item| {
                    let item = server.items.items.get(item)?;
                    Some(item.clone())
                })
                .collect();

            let _ = sender.send(Response::Items {
                list: list.id,
                items,
            });
        }

        Request::CreateItem { list, item } => {
            let Some(mut list) = server.lists.lists.get_mut(&list) else {
                return;
            };

            server.items.items.insert(item.id, item.clone());

            let index = {
                list.items.push(item.id);
                list.items.len() - 1
            };

            server.send_all(
                Some(sender),
                Response::ItemCreated {
                    list: list.id,
                    item,
                    index,
                },
            );
        }

        Request::RemoveItem { list, item: id } => {
            let _ = server.items.items.remove(&id);

            if let Some(mut list) = server.lists.lists.get_mut(&list)
                && let Some(index) = list.items.iter().position(|x| *x == id)
            {
                list.items.remove(index);
                server.send_all(
                    Some(sender),
                    Response::ItemRemoved {
                        list: list.id,
                        item: id,
                        index,
                    },
                );
            }
        }

        Request::RenameItem {
            list,
            item: id,
            name,
        } => {
            if let Some(mut item) = server.items.items.get_mut(&id) {
                item.name = name.clone();
            }

            server.send_all(
                Some(sender),
                Response::ItemRenamed {
                    list,
                    item: id,
                    name,
                },
            );
        }

        Request::CompleteItem {
            list,
            item: id,
            completed,
        } => {
            if let Some(mut item) = server.items.items.get_mut(&id) {
                item.completed = completed;
            }

            server.send_all(
                Some(sender),
                Response::ItemCompleted {
                    list,
                    item: id,
                    completed,
                },
            );
        }
    }
}
