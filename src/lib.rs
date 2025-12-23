use futures::{SinkExt, StreamExt};
use ike::prelude::*;
use indk_proto::v1::{Item, Request, Response};
use reqwest_websocket::{Message, RequestBuilderExt};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use uuid::Uuid;

#[ike::main]
#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    App::install_log();

    let (response_tx, response_rx) = unbounded_channel();
    let (request_tx, mut request_rx) = unbounded_channel();

    tokio::spawn(async move {
        loop {
            if let Err(err) = try_loop(&response_tx, &mut request_rx).await {
                warn!("connection failed with {err:?}");
            }
        }
    });

    request_tx.send(Request::GetItems)?;

    let mut data = Data {
        sender: request_tx,
        receiver: Some(response_rx),
        items: Vec::new(),
    };

    App::new().run(&mut data, ui)?;

    Ok(())
}

async fn try_loop(
    sender: &UnboundedSender<Response>,
    receiver: &mut UnboundedReceiver<Request>,
) -> eyre::Result<()> {
    let cert = reqwest::Certificate::from_pem(include_bytes!("cert.pem"))?;

    let response = reqwest::Client::builder()
        .add_root_certificate(cert)
        .build()?
        .get("wss://91.98.131.126/api/v1/ws")
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
                        sender.send(response)?;
                    }
                } else {
                    return Ok(());
                }

            }
        }
    }
}

struct Data {
    sender: UnboundedSender<Request>,
    receiver: Option<UnboundedReceiver<Response>>,
    items: Vec<Item>,
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    let view = vstack((input(data), flex(items(data)), remove_completed())).align(Align::Fill);
    let view = container(view).padding(0.0).corner_radius(0.0);
    let view = pad([40.0, 10.0, 60.0, 10.0], view);

    provide(
        |_| TextTheme {
            font_size: 18.0,
            ..Default::default()
        },
        effects((window(view), receive())),
    )
}

fn receive() -> impl Effect<Data> + use<> {
    task(
        |data: &mut Data, sink| {
            let mut receiver = data.receiver.take().unwrap();

            async move {
                while let Some(response) = receiver.recv().await {
                    sink.send(response);
                }
            }
        },
        |data: &mut Data, response: Response| match response {
            Response::Items(items) => {
                data.items = items;
            }

            Response::ItemCreated { item, index } => {
                data.items.insert(index, item);
            }

            Response::ItemRemoved { id, .. } => {
                if let Some(index) = data.items.iter().position(|i| i.id == id) {
                    data.items.remove(index);
                }
            }

            Response::ItemRenamed { id, name } => {
                if let Some(item) = data.items.iter_mut().find(|i| i.id == id) {
                    item.name = name;
                }
            }

            Response::ItemCompleted { id, completed } => {
                if let Some(item) = data.items.iter_mut().find(|i| i.id == id) {
                    item.completed = completed;
                }
            }
        },
    )
}

fn input(_data: &mut Data) -> impl View<Data> + use<> {
    entry()
        .placeholder("Hvad mangler vi?")
        .submit_behaviour(SubmitBehaviour {
            keep_focus: true,
            clear_text: true,
        })
        .on_submit(|data: &mut Data, text| {
            let item = Item {
                id: Uuid::new_v4(),
                name: text,
                completed: false,
            };

            let _ = data.sender.send(Request::CreateItem(item.clone()));
            data.items.push(item);
        })
        .padding(12.0)
        .corner_radius(0.0)
        .border_width([0.0, 0.0, 1.0, 0.0])
}

fn items(data: &mut Data) -> impl View<Data> + use<> {
    let complete = data
        .items
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, i)| i.completed);

    let items = data
        .items
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, i)| !i.completed)
        .chain(complete)
        .map(|(index, item)| self::item(index, item))
        .collect::<Vec<_>>();

    vscroll(vstack(items)).bar_border_width([0.0, 0.0, 0.0, 1.0])
}

fn item(index: usize, item: &Item) -> impl View<Data> + use<> {
    container(
        hstack((
            item_completed(index, item.completed),
            flex(item_name(index, &item.name)),
            remove_item(index),
        ))
        .gap(10.0),
    )
    .border_width([0.0, 0.0, 1.0, 0.0])
    .corner_radius(0.0)
}

fn item_name(index: usize, name: &str) -> impl View<Data> + use<> {
    entry()
        .text(name)
        .border_width(0.0)
        .background_color(Color::TRANSPARENT)
        .padding(4.0)
        .on_change(move |data: &mut Data, text| {
            let item = &mut data.items[index];
            item.name = text;

            let _ = data.sender.send(Request::RenameItem {
                id: item.id,
                name: item.name.clone(),
            });
        })
}

fn item_completed(index: usize, completed: bool) -> impl View<Data> + use<> {
    button(
        using_or_default(move |_, palette: &Palette| {
            let color = if completed {
                palette.success
            } else {
                Color::TRANSPARENT
            };

            size(
                [20.0, 20.0],
                picture(Fit::Fill, include_svg!("check.svg")).color(color),
            )
        }),
        move |data: &mut Data| {
            let item = &mut data.items[index];
            item.completed = !item.completed;

            let _ = data.sender.send(Request::CompleteItem {
                id: item.id,
                completed: item.completed,
            });
        },
    )
    .padding(4.0)
}

fn remove_item(index: usize) -> impl View<Data> + use<> {
    button(
        using_or_default(|_, palette: &Palette| {
            size(
                [20.0, 20.0],
                picture(Fit::Fill, include_svg!("xmark.svg")).color(palette.danger),
            )
        }),
        move |data: &mut Data| {
            let item = data.items.remove(index);
            let _ = data.sender.send(Request::RemoveItem(item.id));
        },
    )
    .padding(4.0)
}

fn remove_completed() -> impl View<Data> + use<> {
    using_or_default(|_, palette: &Palette| {
        button(
            center(label("Slet handlede").color(palette.surface)),
            |data: &mut Data| {
                data.items.retain(|item| {
                    if item.completed {
                        let _ = data.sender.send(Request::RemoveItem(item.id));
                    }

                    !item.completed
                });
            },
        )
        .padding(16.0)
        .corner_radius(0.0)
        .border_width([1.0, 0.0, 0.0, 0.0])
        .color(palette.success)
    })
}
