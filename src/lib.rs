use ike::prelude::*;

#[ike::main]
pub fn main() {
    let mut data = Data { items: Vec::new() };
    App::new().run(&mut data, ui);
}

struct Data {
    items: Vec<Item>,
}

struct Item {}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    let view = vstack((input(data), flex(items(data)))).align(Align::Fill);

    window(pad([40.0, 10.0, 40.0, 10.0], view))
}

fn input(_data: &mut Data) -> impl View<Data> + use<> {
    entry().submit_behaviour(SubmitBehaviour {
        keep_focus: true,
        clear_text: true,
    })
}

fn items(_data: &mut Data) -> impl View<Data> + use<> {
    vscroll(vstack(())).bar_border_width(0.0)
}
