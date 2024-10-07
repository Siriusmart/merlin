use serde::{Deserialize, Serialize};

use crate::CollectionItem;

#[derive(Serialize, Deserialize, Clone)]
pub struct CoordsEntry {
    id: i64,
    r#type: String,
    subtype: String,
    access_level: String,
    coords: Coords,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Coords {
    x: i64,
    y: i64,
    z: i64,
}

impl CollectionItem<i64> for CoordsEntry {
    fn id(&self) -> i64 {
        self.id
    }
}
