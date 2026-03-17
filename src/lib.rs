use futures::{SinkExt, StreamExt};
use indk_proto::v1::{Item, Request, Response};
use ori_native::prelude::*;
use reqwest_websocket::{Message, Upgrade};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use uuid::Uuid;

#[ori_native::main]
pub fn main() -> eyre::Result<()> {
    App::init_log();

    let mut data = Data {
        sender: None,
        items: Vec::new(),
    };

    App::new().run(&mut data, ui)?;

    Ok(())
}

async fn try_loop(
    sender: &Sink<Response>,
    receiver: &mut UnboundedReceiver<Request>,
) -> eyre::Result<()> {
    let cert = reqwest::Certificate::from_pem(include_bytes!("cert.pem"))?;

    let response = reqwest::Client::builder()
        .add_root_certificate(cert)
        .http1_only()
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
                        sender.send(response);
                    }
                } else {
                    return Ok(());
                }
            }
        }
    }
}

struct Data {
    sender: Option<UnboundedSender<Request>>,
    items: Vec<Item>,
}

impl Data {
    fn request(&self, request: Request) {
        if let Some(ref sender) = self.sender {
            let _ = sender.send(request);
        }
    }
}

mod theme {
    pub use ori_native::Color;

    pub static BACKGROUND: Color = Color::hex("#f5f7ff");
    pub static CONTRAST: Color = Color::hex("#0a0a0a");
    pub static OUTLINE: Color = Color::BLACK.fade(0.2);
    pub static PRIMARY: Color = Color::hex("#a6d189");
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    let view = column(
        safe_area(
            column((input(data), items(data).flex(1.0), remove_completed()))
                .flex(1.0)
                .gap(12.0),
        )
        .min_height(0.0)
        .flex(1.0),
    )
    .background_color(theme::BACKGROUND)
    .padding(10.0)
    .flex(1.0);

    effects((
        window(view)
            .status_bar(StatusBar {
                color: Some(theme::BACKGROUND),
                light: true,
                ..Default::default()
            })
            .navigation_bar(NavigationBar {
                color: Some(theme::BACKGROUND),
                light: true,
            }),
        receive(),
    ))
}

fn receive() -> impl Effect<Data> + use<> {
    task(
        |data: &mut Data, sink| {
            let (sender, mut receiver) = unbounded_channel();
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

fn input(_data: &Data) -> impl View<Data> + use<> {
    with(
        |_| String::new(),
        |state, _data| {
            column(
                textinput()
                    .text(state)
                    .size(18.0)
                    .newline(Newline::None)
                    .accept_tab(false)
                    .color(theme::CONTRAST)
                    .placeholder("Hvad mangler vi?")
                    .placeholder_color(theme::CONTRAST.fade(0.6))
                    .on_submit(|(state, data): &mut (String, Data), text| {
                        state.clear();

                        let item = Item {
                            id: Uuid::new_v4(),
                            name: text,
                            completed: false,
                        };

                        data.request(Request::CreateItem(item.clone()));
                        data.items.push(item);
                    }),
            )
            .background_color(theme::BACKGROUND.darken(0.04))
            .corner(20.0)
            .padding(20.0)
        },
    )
}

fn items(data: &Data) -> impl View<Data> + Layout + use<> {
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
        .map(|(index, item)| (item.id, self::item(index, item)));

    column(vscroll(column(keyed(items))).flex(1.0))
        .background_color(Color::BLACK.fade(0.05))
        .corner(20.0)
        .overflow(Overflow::Hidden)
        .min_height(0.0)
}

fn item(index: usize, item: &Item) -> impl View<Data> + use<> {
    row((
        item_completed(index, item.completed),
        item_name(index, &item.name).flex(1.0),
        remove_item(index),
    ))
    .align_items(Align::Center)
    .padding(12.0)
    .gap(10.0)
}

fn item_name(index: usize, name: &str) -> impl View<Data> + Layout + use<> {
    textinput()
        .text(name)
        .size(18.0)
        .color(theme::CONTRAST)
        .newline(Newline::None)
        .accept_tab(false)
        .on_change(move |data: &mut Data, text| {
            let item = &mut data.items[index];
            item.name = text;

            let request = Request::RenameItem {
                id: item.id,
                name: item.name.clone(),
            };

            data.request(request);
        })
}

fn item_completed(index: usize, completed: bool) -> impl View<Data> + use<> {
    let color = if completed {
        theme::PRIMARY
    } else {
        Color::TRANSPARENT
    };

    pressable(move |_, _| {
        row(image(include_bytes!("check.svg"))
            .tint(color)
            .size(20.0, 20.0))
        .border_color(theme::OUTLINE)
        .padding(4.0)
        .border(1.0)
        .corner(8.0)
    })
    .on_press(move |data: &mut Data| {
        let item = &mut data.items[index];
        item.completed = !item.completed;

        let request = Request::CompleteItem {
            id: item.id,
            completed: item.completed,
        };

        data.request(request);
    })
}

fn remove_item(index: usize) -> impl View<Data> + use<> {
    pressable(|_, _| {
        row(image(include_bytes!("xmark.svg"))
            .tint(Color::RED.fade(0.7))
            .size(28.0, 28.0))
        .padding(4.0)
        .border(1.0)
        .corner(8.0)
    })
    .on_press(move |data: &mut Data| {
        let item = data.items.remove(index);
        data.request(Request::RemoveItem(item.id));
    })
}

fn remove_completed() -> impl View<Data> + use<> {
    pressable(|state, _| {
        let color = match state.pressed {
            true => theme::PRIMARY.darken(0.1),
            false => theme::PRIMARY,
        };

        transition(color, Ease(0.1), |color, _| {
            row(text("Slet handlede")
                .color(Color::BLACK.fade(0.8))
                .size(18.0))
            .background_color(color)
            .padding(20.0)
            .corner(20.0)
            .justify_contents(Justify::Center)
            .align_items(Align::Center)
        })
    })
    .on_press(|data: &mut Data| {
        data.items.retain(|item| {
            if item.completed
                && let Some(ref sender) = data.sender
            {
                let _ = sender.send(Request::RemoveItem(item.id));
            }

            !item.completed
        });
    })
}
