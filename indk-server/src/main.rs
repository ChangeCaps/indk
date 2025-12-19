mod v1;

use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};

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

    let config = RustlsConfig::from_pem_file("cert.pem", "key.pem").await?;

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
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Default)]
struct ServerState {
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
                let items = DashMap::new();
                let mut order = Vec::new();

                for item in data.items {
                    order.push(item.id);
                    items.insert(item.id, item);
                }

                Ok(Self {
                    senders: RwLock::new(Vec::new()),
                    items: Items {
                        items,
                        order: RwLock::new(order),
                    },
                })
            }
        }
    }

    async fn store(&self, path: impl AsRef<Path>) -> eyre::Result<()> {
        let data = Data::V1(v1::Data {
            items: self.get_items(),
        });

        let json = serde_json::to_string(&data)?;
        tokio::fs::write(path, json).await?;

        Ok(())
    }

    fn get_items(&self) -> Vec<indk_proto::v1::Item> {
        let mut items = Vec::new();

        for item in self.items.order.read().iter() {
            if let Some(item) = self.items.items.get(item) {
                items.push(item.clone());
            }
        }

        items
    }
}

#[derive(Default)]
struct Items {
    items: DashMap<Uuid, indk_proto::v1::Item>,
    order: RwLock<Vec<Uuid>>,
}

#[derive(Serialize, Deserialize)]
enum Data {
    V1(v1::Data),
}
