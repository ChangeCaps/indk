use indk_proto::v1::{List, Request};
use ori_native::prelude::*;
use uuid::Uuid;

use crate::{Global, ListData, Page, theme};

pub fn lists(global: &Global) -> impl View<Global> + Layout + use<> {
    let lists = global
        .lists
        .iter()
        .enumerate()
        .map(|(i, _)| list(i))
        .collect::<Vec<_>>();

    column((
        column(
            text("Lister")
                .family("Inter")
                .size(40.0)
                .color(theme::CONTRAST.fade(0.8)),
        )
        .border_bottom(2.0)
        .border_color(theme::CONTRAST.fade(0.8)),
        lists,
        pressable(|_, _| {
            row((
                image(include_bytes!("../icon/plus.svg")).size(32.0, 32.0),
                text("tilføj").size(18.0).color(theme::CONTRAST.fade(0.8)),
            ))
            .padding_left(20.0)
            .align_items(Align::Center)
            .gap(10.0)
        })
        .on_press(|global: &mut Global| {
            let list = List {
                id: Uuid::new_v4(),
                name: String::from("ny liste"),
            };

            global.lists.push(list.clone());

            Action::message(Request::CreateList { list }, None).with_rebuild(true)
        }),
    ))
    .align_items(Align::Stretch)
    .gap(10.0)
}

fn list(index: usize) -> impl View<Global> + use<> {
    with(
        |_| false,
        move |edit, global: &Global| {
            let list = &global.lists[index];

            let name = match edit {
                true => any(textinput()
                    .text(&list.name)
                    .size(18.0)
                    .color(theme::CONTRAST)
                    .flex(1.0)
                    .on_change(move |(_, global): &mut (_, Global), text| {
                        global.lists[index].name = text;

                        Action::message(
                            Request::RenameList {
                                list: global.lists[index].id,
                                name: global.lists[index].name.clone(),
                            },
                            None,
                        )
                        .with_rebuild(true)
                    })),

                false => any(pressable(move |(_, global): &(_, Global), _| {
                    let list = &global.lists[index];
                    text(&list.name).size(18.0).color(theme::CONTRAST).flex(1.0)
                })
                .on_press(move |(_, global)| {
                    Action::new()
                        .with_message(
                            Page::List(ListData {
                                id: global.lists[index].id,
                                items: Vec::new(),
                                is_menu_open: false,
                            }),
                            None,
                        )
                        .with_message(
                            Request::GetItems {
                                list: global.lists[index].id,
                            },
                            None,
                        )
                })),
            };

            row((
                name,
                pressable(|(edit, _): &(bool, _), _| {
                    let color = match edit {
                        true => theme::PRIMARY,
                        false => theme::CONTRAST,
                    };

                    image(include_bytes!("../icon/edit.svg"))
                        .tint(color)
                        .size(32.0, 32.0)
                })
                .on_press(|(edit, _): &mut (bool, _)| *edit = !*edit),
            ))
            .justify_contents(Justify::SpaceBetween)
            .align_items(Align::Center)
        },
    )
}
