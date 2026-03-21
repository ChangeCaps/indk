use indk_proto::v1::{Item, Request};
use ori_native::prelude::*;
use uuid::Uuid;

use crate::{Global, ListData, theme, ui};

pub fn page(data: &ListData) -> impl View<(Global, ListData)> + use<> {
    column(transition(
        data.is_menu_open as i32 as f32,
        BackInOut(0.8),
        |(_, data), t| {
            let menu = (t > 0.0).then(|| menu(t));
            flex((page_contents(data), menu)).flex(1.0).min_height(0.0)
        },
    ))
    .background(theme::BACKGROUND)
    .flex(1.0)
}

fn page_contents(data: &ListData) -> impl View<(Global, ListData)> + use<> {
    safe_area(
        column((
            row((input(), menu_button())).gap(10.0),
            items(data).flex(1.0),
            remove_completed(),
        ))
        .min_height(0.0)
        .padding(10.0)
        .flex(1.0)
        .gap(12.0),
    )
    .flex(1.0)
}

fn menu(t: f32) -> impl View<(Global, ListData)> + use<> {
    pressable(move |_, _| {
        row(pressable(move |_, _| {
            column(vscroll(menu_contents()).flex(1.0))
                .left(Fract(t * 0.7 - 1.0))
                .width(Fract(1.0))
                .padding_left(Fract(0.3))
                .background(theme::BACKGROUND)
                .shadow_color(Color::BLACK.fade(0.4))
                .shadow_radius(50.0)
                .corner(20.0)
        })
        .on_press(|_| info!("click")))
        .background(Color::BLACK.fade(0.2 * t))
        .position(Position::Absolute)
        .inset(0.0)
    })
    .on_press(|(_, data): &mut (_, ListData)| data.is_menu_open = false)
}

fn menu_contents() -> impl View<(Global, ListData)> + use<> {
    using_or_default(|(global, _): &(Global, _), insets: &SafeAreaInsets| {
        column(map(ui::lists::lists(global), |(global, _), map| {
            map(global)
        }))
        .padding_left(insets.left + 40.0)
        .padding_top(insets.top + 40.0)
        .padding_bottom(insets.bottom)
        .padding_right(40.0)
    })
}

fn menu_button() -> impl View<(Global, ListData)> + use<> {
    pressable(|_, state| {
        let color = match state.pressed {
            true => theme::PRIMARY.darken(0.1),
            false => theme::PRIMARY,
        };

        transition(color, Ease(0.1), |_, color| {
            row(image(include_bytes!("../icon/menu.svg"))
                .size(28.0, 28.0)
                .tint(theme::CONTRAST.fade(0.8)))
            .background(color.fade(0.5))
            .padding(20.0)
            .corner(20.0)
            .justify_contents(Justify::Center)
            .align_items(Align::Center)
        })
    })
    .on_press(|(_, data): &mut (_, ListData)| data.is_menu_open = true)
}

fn input() -> impl View<(Global, ListData)> + use<> {
    with(
        |_| String::new(),
        |state, _| {
            column(
                textinput()
                    .text(state)
                    .size(18.0)
                    .newline(Newline::None)
                    .accept_tab(false)
                    .color(theme::CONTRAST)
                    .placeholder("Hvad mangler vi?")
                    .placeholder_color(theme::CONTRAST.fade(0.6))
                    .on_submit(|(state, (_, data)): &mut (String, (_, ListData)), text| {
                        state.clear();

                        let item = Item {
                            id: Uuid::new_v4(),
                            name: text,
                            completed: false,
                        };

                        data.items.push(item.clone());
                        Action::message(
                            Request::CreateItem {
                                list: data.id,
                                item,
                            },
                            None,
                        )
                        .with_rebuild(true)
                    }),
            )
            .background(theme::BACKGROUND.darken(0.04))
            .corner(20.0)
            .padding(20.0)
            .flex(1.0)
        },
    )
}

fn items(data: &ListData) -> impl View<(Global, ListData)> + Layout + use<> {
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
        .background(Color::BLACK.fade(0.05))
        .corner(20.0)
        .overflow(Overflow::Hidden)
}

fn item(index: usize, item: &Item) -> impl View<(Global, ListData)> + use<> {
    row((
        item_completed(index, item.completed),
        item_name(index, &item.name).flex(1.0),
        remove_item(index),
    ))
    .align_items(Align::Center)
    .padding(12.0)
    .gap(10.0)
}

fn item_name(index: usize, name: &str) -> impl View<(Global, ListData)> + Layout + use<> {
    textinput()
        .text(name)
        .size(18.0)
        .color(theme::CONTRAST)
        .newline(Newline::None)
        .accept_tab(false)
        .on_change(move |(_, data): &mut (_, ListData), text| {
            let item = &mut data.items[index];
            item.name = text;

            Action::message(
                Request::RenameItem {
                    list: data.id,
                    item: item.id,
                    name: item.name.clone(),
                },
                None,
            )
            .with_rebuild(true)
        })
}

fn item_completed(index: usize, completed: bool) -> impl View<(Global, ListData)> + use<> {
    let color = if completed {
        theme::PRIMARY
    } else {
        Color::TRANSPARENT
    };

    pressable(move |_, _| {
        row(image(include_bytes!("../icon/check.svg"))
            .tint(color)
            .size(20.0, 20.0))
        .border_color(theme::OUTLINE)
        .padding(4.0)
        .border(1.0)
        .corner(8.0)
    })
    .on_press(move |(_, data): &mut (_, ListData)| {
        let item = &mut data.items[index];
        item.completed = !item.completed;

        Action::message(
            Request::CompleteItem {
                list: data.id,
                item: item.id,
                completed: item.completed,
            },
            None,
        )
        .with_rebuild(true)
    })
}

fn remove_item(index: usize) -> impl View<(Global, ListData)> + use<> {
    pressable(|_, _| {
        row(image(include_bytes!("../icon/xmark.svg"))
            .tint(Color::RED.fade(0.7))
            .size(28.0, 28.0))
        .padding(4.0)
        .border(1.0)
        .corner(8.0)
    })
    .on_press(move |(_, data): &mut (_, ListData)| {
        let item = data.items.remove(index);
        Action::message(
            Request::RemoveItem {
                list: data.id,
                item: item.id,
            },
            None,
        )
        .with_rebuild(true)
    })
}

fn remove_completed() -> impl View<(Global, ListData)> + use<> {
    pressable(|_, state| {
        let color = match state.pressed {
            true => theme::PRIMARY.darken(0.1),
            false => theme::PRIMARY,
        };

        transition(color, Ease(0.1), |_, color| {
            row(text("Slet handlede")
                .color(Color::BLACK.fade(0.8))
                .size(18.0))
            .background(color)
            .padding(20.0)
            .corner(20.0)
            .justify_contents(Justify::Center)
            .align_items(Align::Center)
        })
    })
    .on_press(|(_, data): &mut (_, ListData)| {
        let mut action = Action::rebuild();

        data.items.retain(|item| {
            if item.completed {
                action.add_message(
                    Request::RemoveItem {
                        list: data.id,
                        item: item.id,
                    },
                    None,
                );
            }

            !item.completed
        });

        action
    })
}
