use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    GetLists {},

    CreateList {
        list: List,
    },

    RemoveList {
        list: Uuid,
    },

    RenameList {
        list: Uuid,
        name: String,
    },

    GetItems {
        list: Uuid,
    },

    CreateItem {
        list: Uuid,
        item: Item,
    },

    RemoveItem {
        list: Uuid,
        item: Uuid,
    },

    RenameItem {
        list: Uuid,
        item: Uuid,
        name: String,
    },

    CompleteItem {
        list: Uuid,
        item: Uuid,
        completed: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    Lists {
        lists: Vec<List>,
    },

    ListCreated {
        list: List,
        index: usize,
    },

    ListRemoved {
        list: Uuid,
    },

    ListRenamed {
        list: Uuid,
        name: String,
    },

    Items {
        list: Uuid,
        items: Vec<Item>,
    },

    ItemCreated {
        list: Uuid,
        item: Item,
        index: usize,
    },

    ItemRemoved {
        list: Uuid,
        item: Uuid,
        index: usize,
    },

    ItemRenamed {
        list: Uuid,
        item: Uuid,
        name: String,
    },

    ItemCompleted {
        list: Uuid,
        item: Uuid,
        completed: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub completed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List {
    pub id: Uuid,
    pub name: String,
}
