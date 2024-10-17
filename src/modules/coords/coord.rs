use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{modules::coords::collection::COORDS, CollectionItem, Counter, Mongo};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Coord {
    #[serde(rename = "_id")]
    pub id: i64,
    pub cog: i64,
    pub subcog: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub x: i64,
    pub z: i64,
    pub author_id: u64,
}

impl CollectionItem<i64> for Coord {
    fn id(&self) -> i64 {
        self.id
    }
}

impl Coord {
    pub async fn new(
        display_name: String,
        description: String,
        author_id: u64,
        cog: i64,
        subcog: i64,
        x: i64,
        z: i64,
    ) -> Result<Self, &'static str> {
        let name = display_name.replace(' ', "-").to_lowercase();

        if name.chars().any(|c| !c.is_alphanumeric() && c != '-') {
            return Err("name contains illegal characters");
        }

        let coords = unsafe { COORDS.get() }.unwrap();

        if coords
            .find_one(doc! {"name": &name})
            .await
            .unwrap()
            .is_some()
        {
            return Err("a coord entry with that name already exists");
        }

        let new = Self {
            id: Counter::bump_get("coords-coords", Mongo::database())
                .await
                .unwrap(),
            cog,
            subcog,
            name,
            display_name,
            author_id,
            description,
            z,
            x,
        };

        new.save_create(unsafe { COORDS.get() }.unwrap())
            .await
            .unwrap();

        Ok(new)
    }
}
