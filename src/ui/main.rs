use ori_native::prelude::*;

use crate::{Data, theme, ui};

pub fn page(data: &Data) -> impl View<Data> + use<> {
    column(map(
        ui::lists::lists(&data.global).width(300.0),
        |data: &mut Data, map| map(&mut data.global),
    ))
    .background(theme::BACKGROUND)
    .justify_content(Justify::Center)
    .align_items(Align::Center)
    .flex(1.0)
}
