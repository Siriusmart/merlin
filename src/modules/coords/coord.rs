use std::{collections::HashMap, fmt::Display};

use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{Context, Message},
    futures::StreamExt,
};

use crate::{
    modules::coords::{
        category::Category,
        collection::{CATEGORIES, COORDS},
    },
    Clearance, CollectionItem, Counter, Mongo,
};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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
    pub dim: Dimension,
    pub added: i64,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl CollectionItem<i64> for Coord {
    fn id(&self) -> i64 {
        self.id
    }
}

impl Coord {
    pub async fn find_by_name(name: &str) -> Option<Self> {
        unsafe { COORDS.get() }
            .unwrap()
            .find_one(doc! { "name": &name })
            .await
            .unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        display_name: String,
        description: String,
        author_id: u64,
        cog: i64,
        subcog: i64,
        x: i64,
        z: i64,
        dim: Dimension,
        tags: Vec<String>,
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
            added: chrono::Utc::now().timestamp(),
            tags,
        };

        new.save_create(unsafe { COORDS.get() }.unwrap())
            .await
            .unwrap();

        Ok(new)
    }

    pub async fn is_allowed(
        &self,
        ctx: &Context,
        msg: &Message,
        lookup: &mut HashMap<(i64, i64), (bool, String, String)>,
    ) -> bool {
        if let Some((b, ..)) = lookup.get(&(self.cog, self.subcog)) {
            return *b;
        }

        async fn is_allowed_core(
            entry: &Coord,
            ctx: &Context,
            msg: &Message,
        ) -> (bool, String, String) {
            match (entry.cog, entry.subcog) {
                (0, 0) => (
                    true,
                    "generic.unspecified".to_string(),
                    "generic.unspecified".to_string(),
                ),
                (0, 1) => (
                    entry.author_id == msg.author.id.get(),
                    "generic.private".to_string(),
                    "generic.private".to_string(),
                ),
                (cog, subcog) => {
                    let cog = if let Some(cog) =
                        Category::find_by_id(cog, unsafe { CATEGORIES.get() }.unwrap())
                            .await
                            .unwrap()
                    {
                        cog
                    } else {
                        return (false, String::new(), String::new());
                    };

                    let mut allowed = Clearance::is_allowed(&cog.allowed, ctx, msg)
                        .await
                        .unwrap_or(true);

                    let mut subcog_display = None;
                    let mut subcog_name = None;

                    if let Some(subcog) = cog.subcategories.get(&subcog.to_string()) {
                        allowed &= Clearance::is_allowed(&subcog.allowed, ctx, msg)
                            .await
                            .unwrap_or(true);
                        subcog_display = Some(subcog.display_name.as_str());
                        subcog_name = Some(subcog.name.as_str());
                    }

                    (
                        allowed,
                        if allowed {
                            format!(
                                "{}.{}",
                                cog.display_name,
                                subcog_display.unwrap_or("unspecified")
                            )
                        } else {
                            String::new()
                        },
                        if allowed {
                            format!("{}.{}", cog.name, subcog_name.unwrap_or("unspecified"))
                        } else {
                            String::new()
                        },
                    )
                }
            }
        }

        let (allowed, display, name) = is_allowed_core(self, ctx, msg).await;
        lookup.insert((self.cog, self.subcog), (allowed, display, name));
        allowed
    }

    pub async fn find_near(
        x: i64,
        z: i64,
        r: u64,
        dim: Dimension,
        ctx: &Context,
        msg: &Message,
    ) -> Option<Coord> {
        let r2 = r.pow(2) as i64;

        let mut cursor = unsafe { COORDS.get() }
            .unwrap()
            .find(Document::new())
            .await
            .unwrap();

        // allowed, display_name, name
        let mut clearance_lookup: HashMap<(i64, i64), (bool, String, String)> = HashMap::new();

        while let Some(coord) = cursor.next().await {
            let coord = coord.unwrap();

            if !coord.is_allowed(ctx, msg, &mut clearance_lookup).await {
                continue;
            }

            if (x - coord.x).pow(2) + (z - coord.z).pow(2) <= r2 && coord.dim == dim {
                return Some(coord);
            }
        }

        None
    }
}
