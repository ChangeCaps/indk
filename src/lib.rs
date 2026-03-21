use std::env;

use futures::{SinkExt, StreamExt};
use indk_proto::v1::{Item, List, Request, Response};
use ori_native::prelude::*;
use reqwest_websocket::{Message, Upgrade};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use uuid::Uuid;

mod ui;

#[ori_native::main]
pub fn main() -> eyre::Result<()> {
    App::init_log();

    let mut data = Data {
        page: Page::Main,
        global: Global { lists: Vec::new() },
        sender: None,
    };

    App::new().run(&mut data, ui)?;

    Ok(())
}

async fn try_loop(
    sink: &Sink<Response>,
    receiver: &mut UnboundedReceiver<Request>,
) -> eyre::Result<()> {
    let api_url = env::var("INDK_API").unwrap_or_else(|_| String::from("wss://91.98.131.126"));

    let cert = reqwest::Certificate::from_pem(include_bytes!("cert.pem"))?;

    let response = reqwest::Client::builder()
        .add_root_certificate(cert)
        .http1_only()
        .build()?
        .get(format!("{api_url}/api/v1/ws"))
        .upgrade()
        .send()
        .await?;

    let mut websocket = response.into_websocket().await?;

    loop {
        tokio::select! {
            request = receiver.recv() => {
                if let Some(request) = request {
                    let json = Message::text_from_json(&request)?;
                    websocket.send(json).await?;
                } else {
                    return Ok(());
                }
            }

            message = websocket.next() => {
                if let Some(message) = message {
                    if let Ok(response) = message?.json() {
                        sink.send(response);
                    }
                } else {
                    return Ok(());
                }
            }
        }
    }
}

struct Data {
    page: Page,
    global: Global,
    sender: Option<UnboundedSender<Request>>,
}

struct Global {
    lists: Vec<List>,
}

enum Page {
    Main,
    List(ListData),
}

struct ListData {
    id: Uuid,
    items: Vec<Item>,
    is_menu_open: bool,
}

mod theme {
    pub use ori_native::Color;

    pub static BACKGROUND: Color = Color::hex("#f5f7ff");
    pub static CONTRAST: Color = Color::hex("#0a0a0a");
    pub static OUTLINE: Color = Color::BLACK.fade(0.2);
    pub static PRIMARY: Color = Color::hex("#a6d189");
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    let contents = match data.page {
        Page::Main => any(ui::main::page(data)),

        Page::List(ref data) => any(map_with(ui::list::page(data), |data: &mut Data, map| {
            if let Page::List(ref mut list_data) = data.page {
                map(&mut data.global, list_data);
            }
        })),
    };

    effects((
        window(contents)
            .status_bar(StatusBar {
                color: Some(Color::TRANSPARENT),
                light: true,
                ..Default::default()
            })
            .navigation_bar(NavigationBar {
                color: Some(theme::BACKGROUND),
                light: true,
            }),
        responses(),
        receive(|data: &mut Data, request: Request| {
            if let Some(ref sender) = data.sender {
                let _ = sender.send(request);
            }

            Action::new()
        }),
        receive(|data: &mut Data, page: Page| {
            data.page = page;
        }),
    ))
}

fn responses() -> impl Effect<Data> + use<> {
    task(
        |data: &mut Data, sink| {
            let (sender, mut receiver) = unbounded_channel();
            let _ = sender.send(Request::GetLists {});
            data.sender = Some(sender);

            async move {
                loop {
                    if let Err(err) = try_loop(&sink, &mut receiver).await {
                        warn!("connection failed with {err:?}");
                    }
                }
            }
        },
        |data: &mut Data, _, response: Response| match response {
            Response::Lists { lists } => {
                data.global.lists = lists;
            }

            Response::ListCreated { list, index } => {
                data.global.lists.insert(index, list);
            }

            Response::ListRemoved { list } => {
                if let Some(index) = data.global.lists.iter().position(|l| l.id == list) {
                    data.global.lists.remove(index);
                }
            }

            Response::ListRenamed { list, name } => {
                if let Some(item) = data.global.lists.iter_mut().find(|l| l.id == list) {
                    item.name = name;
                }
            }

            Response::Items { list, items } => {
                if let Page::List(ref mut data) = data.page
                    && data.id == list
                {
                    data.items = items;
                }
            }

            Response::ItemCreated { list, item, index } => {
                if let Page::List(ref mut data) = data.page
                    && data.id == list
                {
                    data.items.insert(index, item);
                }
            }

            Response::ItemRemoved { list, item, .. } => {
                if let Page::List(ref mut data) = data.page
                    && let Some(index) = data.items.iter().position(|i| i.id == item)
                    && data.id == list
                {
                    data.items.remove(index);
                }
            }

            Response::ItemRenamed { list, item, name } => {
                if let Page::List(ref mut data) = data.page
                    && let Some(item) = data.items.iter_mut().find(|i| i.id == item)
                    && data.id == list
                {
                    item.name = name;
                }
            }

            Response::ItemCompleted {
                list,
                item,
                completed,
            } => {
                if let Page::List(ref mut data) = data.page
                    && let Some(item) = data.items.iter_mut().find(|i| i.id == item)
                    && data.id == list
                {
                    item.completed = completed;
                }
            }
        },
    )
}
