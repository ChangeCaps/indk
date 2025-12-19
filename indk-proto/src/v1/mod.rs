use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    GetItems,
    CreateItem(Item),
    RemoveItem(Uuid),
    RenameItem { id: Uuid, name: String },
    CompleteItem { id: Uuid, completed: bool },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    Items(Vec<Item>),
    ItemCreated { item: Item, index: usize },
    ItemRemoved { id: Uuid, index: usize },
    ItemRenamed { id: Uuid, name: String },
    ItemCompleted { id: Uuid, completed: bool },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub completed: bool,
}
