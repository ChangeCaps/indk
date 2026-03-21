mod v1;

use std::{env, net::SocketAddr, path::Path, sync::Arc, time::Duration};

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let data_path = "data.json";

    let server = ServerState::load(data_path).await.unwrap_or_default();
    let server = Arc::new(server);

    tokio::spawn({
        let server = server.clone();

        async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;

                let _ = server.store(data_path).await;
            }
        }
    });

    let app = Router::new()
        .nest("/api/v1", v1::router().await)
        .with_state(server);

    let addr = SocketAddr::from(([0; 4], 443));

    if env::var("INDK_SERVER_LOCAL").is_ok() {
        axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    } else {
        let config = RustlsConfig::from_pem_file("cert.pem", "key.pem").await?;
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await?;
    }

    Ok(())
}

#[derive(Default)]
struct ServerState {
    lists: Lists,
    items: Items,
    senders: RwLock<Vec<UnboundedSender<indk_proto::v1::Response>>>,
}

impl ServerState {
    fn send_all(
        &self,
        exclude: Option<&UnboundedSender<indk_proto::v1::Response>>,
        response: indk_proto::v1::Response,
    ) {
        for sender in self.senders.read().iter() {
            if exclude.is_some_and(|exclude| exclude.same_channel(sender)) {
                continue;
            }

            let _ = sender.send(response.clone());
        }
    }

    async fn load(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let source = tokio::fs::read_to_string(path).await?;
        let data: Data = serde_json::from_str(&source)?;

        match data {
            Data::V1(data) => {
                let lists = DashMap::new();
                let mut list_order = Vec::new();

                for list in data.lists {
                    list_order.push(list.id);
                    lists.insert(list.id, list);
                }

                let items = data.items.into_iter().map(|item| (item.id, item)).collect();

                Ok(Self {
                    senders: RwLock::new(Vec::new()),
                    lists: Lists {
                        lists,
                        order: RwLock::new(list_order),
                    },
                    items: Items { items },
                })
            }
        }
    }

    async fn store(&self, path: impl AsRef<Path>) -> eyre::Result<()> {
        let data = Data::V1(v1::Data {
            lists: self.get_lists(),
            items: self.get_items(),
        });

        let json = serde_json::to_string(&data)?;
        tokio::fs::write(path, json).await?;

        Ok(())
    }

    fn get_lists(&self) -> Vec<v1::List> {
        let mut lists = Vec::new();

        for list in self.lists.order.read().iter() {
            if let Some(item) = self.lists.lists.get(list) {
                lists.push(item.clone());
            }
        }

        lists
    }

    fn get_items(&self) -> Vec<indk_proto::v1::Item> {
        self.items.items.iter().map(|item| item.clone()).collect()
    }
}

#[derive(Default)]
struct Lists {
    lists: DashMap<Uuid, v1::List>,
    order: RwLock<Vec<Uuid>>,
}

#[derive(Default)]
struct Items {
    items: DashMap<Uuid, indk_proto::v1::Item>,
}

#[derive(Serialize, Deserialize)]
enum Data {
    V1(v1::Data),
}
