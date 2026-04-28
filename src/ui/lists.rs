use indk_proto::v1::{List, Request};
use ori_native::prelude::*;
use uuid::Uuid;

use crate::{Global, ListData, MODAL, Page, theme};

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
        .border_bottom_width(2.0)
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
    with_default(move |state: &Modal, _| {
        let name = pressable(move |(_, global): &(_, Global), _| {
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
        });

        effect(
            row((
                name,
                pressable(|_, _| {
                    image(include_bytes!("../icon/settings.svg"))
                        .tint(theme::CONTRAST)
                        .size(32.0, 32.0)
                })
                .on_press(|(modal, _): &mut (Modal, _)| modal.is_open = true),
            ))
            .justify_content(Justify::SpaceBetween)
            .align_items(Align::Center),
            state.is_open.then(move || {
                let modal = pressable(move |_, _| {
                    column(pressable(move |(_, global): &(_, Global), _| {
                        column((without(name_input(global, index)), remove_button(index)))
                            .background(theme::BACKGROUND)
                            .position(Position::Absolute)
                            .justify_content(Justify::SpaceBetween)
                            .padding(16.0)
                            .gap(10.0)
                            .size(300.0, 400.0)
                            .corner(8.0)
                            .shadow(4.0, 4.0, 12.0, Color::BLACK.fade(0.4))
                    }))
                    .background(Color::BLACK.fade(0.2))
                    .position(Position::Absolute)
                    .inset(0.0)
                    .justify_content(Justify::Center)
                    .align_items(Align::Center)
                })
                .on_press(|(modal, _): &mut (Modal, _)| modal.is_open = false);

                teleport(MODAL, modal)
            }),
        )
    })
}

#[derive(Default)]
struct Modal {
    is_open: bool,
}

fn name_input(global: &Global, index: usize) -> impl View<Global> + use<> {
    let list = &global.lists[index];

    column(
        textinput()
            .text(&list.name)
            .color(theme::CONTRAST)
            .size(22.0)
            .placeholder("Liste navn")
            .newline(Newline::None)
            .accept_tab(false)
            .on_change(move |global: &mut Global, name| {
                let list = &mut global.lists[index];
                list.name = name;

                Action::new()
                    .with_message(
                        Request::RenameList {
                            list: list.id,
                            name: list.name.clone(),
                        },
                        None,
                    )
                    .with_rebuild(true)
            }),
    )
    .border_bottom(1.0, theme::OUTLINE)
    .padding(4.0)
}

fn remove_button(index: usize) -> impl View<(Modal, Global)> {
    text_button(
        "Slet liste",
        move |(modal, global): &mut (Modal, Global)| {
            let list = global.lists.remove(index);
            modal.is_open = false;

            Action::new()
                .with_rebuild(true)
                .with_message(Request::RemoveList { list: list.id }, None)
        },
    )
}

fn text_button<T, A>(
    contents: &'static str,
    on_press: impl FnMut(&mut T) -> A + 'static,
) -> impl View<T>
where
    A: Into<Action>,
{
    pressable(move |_, state| {
        let mut color = theme::CONTRAST;

        if state.pressed {
            color = color.fade(0.5);
        }

        text(contents).size(22.0).color(color)
    })
    .on_press(on_press)
}
