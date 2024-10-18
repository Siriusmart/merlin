use std::fmt::Display;

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{modules::coords::collection::COORDS, CollectionItem, Counter, Mongo};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Dimension {
    #[serde(rename = "ow")]
    Overworld,
    #[serde(rename = "nether")]
    Nether,
    #[serde(rename = "end")]
    End,
}

impl Dimension {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ow" => Some(Self::Overworld),
            "nether" => Some(Self::Nether),
            "end" => Some(Self::End),
            _ => None,
        }
    }
}

impl Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overworld => f.write_str("overworld"),
            Self::Nether => f.write_str("nether"),
            Self::End => f.write_str("end"),
        }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Overworld
    }
}

#[derive(Serialize, Deserialize, Clone)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim: Option<Dimension>,
}

impl CollectionItem<i64> for Coord {
    fn id(&self) -> i64 {
        self.id
    }
}

impl Coord {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        display_name: String,
        description: String,
        author_id: u64,
        cog: i64,
        subcog: i64,
        x: i64,
        z: i64,
        dim: Option<Dimension>,
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
            dim,
        };

        new.save_create(unsafe { COORDS.get() }.unwrap())
            .await
            .unwrap();

        Ok(new)
    }
}
