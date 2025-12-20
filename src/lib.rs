use futures::{SinkExt, StreamExt, stream::SplitStream};
use ike::prelude::*;
use indk_proto::v1::{Item, Request, Response};
use reqwest_websocket::{Message, RequestBuilderExt, WebSocket};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use uuid::Uuid;

#[ike::main]
#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let cert = reqwest::Certificate::from_pem(include_bytes!("cert.pem"))?;

    let response = reqwest::Client::builder()
        .add_root_certificate(cert)
        .build()?
        .get("wss://91.98.131.126/api/v1/ws")
        .upgrade()
        .send()
        .await
        .unwrap();

    let websocket = response.into_websocket().await?;

    let (mut sink, stream) = websocket.split();
    let (sender, mut receiver) = unbounded_channel();

    tokio::spawn(async move {
        while let Some(request) = receiver.recv().await {
            let json = Message::text_from_json(&request).unwrap();
            sink.send(json).await.unwrap();
        }
    });

    sender.send(Request::GetItems)?;

    let mut data = Data {
        sender,
        stream: Some(stream),
        items: Vec::new(),
    };

    App::new().run(&mut data, ui);

    Ok(())
}

struct Data {
    sender: UnboundedSender<Request>,
    stream: Option<SplitStream<WebSocket>>,
    items: Vec<Item>,
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    let view = vstack((input(data), flex(items(data))))
        .align(Align::Fill)
        .gap(10.0);

    provide(
        |_| TextTheme {
            font_size: 18.0,
            ..Default::default()
        },
        effects((
            window(pad([40.0, 10.0, 40.0, 10.0], view)),
            task(
                |data: &mut Data, sink| {
                    let mut receiver = data.stream.take().unwrap();

                    async {
                        tokio::spawn(async move {
                            while let Some(Ok(message)) = receiver.next().await {
                                if let Ok(response) = message.json() {
                                    sink.send(response);
                                }
                            }
                        });
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
            ),
        )),
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

            data.sender.send(Request::CreateItem(item.clone())).unwrap();
            data.items.push(item.clone());
        })
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

    vscroll(vstack(items)).bar_border_width(1.0)
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
    .border_width([0.0, 0.0, 1.0, 1.0])
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
