use std::{collections::HashMap, fmt::Display};

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serenity::all::{Context, Message};

use crate::{
    modules::coords::{
        category::Category,
        collection::{CATEGORIES, COORDS},
    },
    Clearance, CollectionItem, Counter, Mongo,
};

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
}
