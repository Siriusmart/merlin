use std::{collections::HashMap, path::PathBuf};

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serenity::futures::StreamExt;
use tokio::fs;

use crate::{
    modules::coords::collection::{CATEGORIES, COORDS},
    CollectionItem, Counter, Mongo,
};

use super::config::COORDS_CONFIG;

// special categories
// X.0 - uncategorised
// 0.1 - private: author only

#[derive(Serialize, Deserialize, Clone)]
pub struct Category {
    #[serde(rename = "_id")]
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub allowed: Vec<String>,
    pub subcogcounter: i64,
    pub subcategories: HashMap<String, Subcategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Subcategory {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub allowed: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_path: Option<String>,
}

impl CollectionItem<i64> for Category {
    fn id(&self) -> i64 {
        self.id
    }
}

impl Subcategory {
    pub fn new(
        name: String,
        display_name: String,
        description: String,
        id: i64,
        attachment_path: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            display_name,
            description,
            allowed: Default::default(),
            attachment_path,
        }
    }
}

impl Category {
    // category, category id, subcog id
    pub async fn cogs_from_name(name: &str) -> Option<(Option<Category>, i64, Option<i64>)> {
        match name {
            "generic.unspecified" => return Some((None, 0, Some(0))),
            "generic.private" => return Some((None, 0, Some(1))),
            "generic" => return Some((None, 0, None)),
            _ => {}
        }

        if let Some((left, right)) = name.split_once('.') {
            Self::get(left).await.and_then(|cog| {
                let id = cog.id;
                let subcog = if right == "unspecified" {
                    0
                } else {
                    cog.get_subcog(right)?.id
                };
                Some((Some(cog), id, Some(subcog)))
            })
        } else {
            Self::get(name).await.map(|cog| {
                let id = cog.id;
                (Some(cog), id, None)
            })
        }
    }

    pub fn contains(&self, subcog: &str) -> bool {
        self.subcategories.values().any(|item| item.name == subcog)
    }

    pub fn get_subcog(&self, subcog: &str) -> Option<&Subcategory> {
        self.subcategories.values().find(|val| val.name == subcog)
    }

    pub fn get_subcog_mut(&mut self, subcog: &str) -> Option<&mut Subcategory> {
        self.subcategories
            .values_mut()
            .find(|val| val.name == subcog)
    }

    pub async fn get(display_name: &str) -> Option<Self> {
        let name = display_name.replace(' ', "-").to_lowercase();

        let categories = unsafe { CATEGORIES.get() }.unwrap();

        categories.find_one(doc! {"name": &name}).await.unwrap()
    }

    pub async fn new(
        display_name: String,
        description: String,
        attachment_path: Option<String>,
    ) -> Result<Self, &'static str> {
        let name = display_name.replace(' ', "-").to_lowercase();

        if name.chars().any(|c| !c.is_alphanumeric() && c != '-') {
            return Err("name contains illegal characters");
        }

        let categories = unsafe { CATEGORIES.get() }.unwrap();

        if categories
            .find_one(doc! {"name": &name})
            .await
            .unwrap()
            .is_some()
        {
            return Err("a category with that name already exists");
        }

        let out = Category {
            id: Counter::bump_get("coords-categories", Mongo::database())
                .await
                .unwrap(),
            name,
            display_name,
            description,
            allowed: vec!["?coordmod".to_string()],
            subcogcounter: 1,
            subcategories: HashMap::new(),
            attachment_path,
        };

        out.save_create(categories).await.unwrap();

        Ok(out)
    }

    // returns false if the from dir contains folders, but a target dir is not specified
    pub async fn move_all(cog: &Category, subcog: Option<i64>, from: &str, to: &str) {
        let from_dir = PathBuf::from(from);
        let to_dir = PathBuf::from(to);

        let mut filter = doc! { "cog": cog.id };
        if let Some(subcog) = subcog {
            filter.insert("subcog", subcog);
        }

        let mut queue = Vec::new();
        let mut cursor = unsafe { COORDS.get() }.unwrap().find(filter).await.unwrap();

        while let Some(coord) = cursor.next().await {
            let coord = coord.unwrap();
            let from = from_dir.join(coord.id.to_string());

            if subcog.is_none()
                && cog
                    .subcategories
                    .get(&coord.subcog.to_string())
                    .is_some_and(|subcog| subcog.attachment_path.is_some())
            {
                continue;
            }

            if fs::try_exists(&from).await.unwrap() {
                let to = to_dir.join(coord.id.to_string());
                if to == from {
                    continue;
                }
                queue.push((from, to));
            }
        }

        if !queue.is_empty() {
            if !fs::try_exists(&to_dir).await.unwrap() {
                fs::create_dir_all(to_dir).await.unwrap();
            }

            for (from, to) in queue {
                fs::rename(from, to).await.unwrap();
            }

            if fs::read_dir(&from_dir)
                .await
                .unwrap()
                .next_entry()
                .await
                .unwrap()
                .is_none()
            {
                fs::remove_dir(from_dir).await.unwrap();
            }
        }
    }

    pub async fn path(cog: i64, subcog: i64) -> PathBuf {
        let cog = if let Some(cog) = Category::find_by_id(cog, unsafe { CATEGORIES.get() }.unwrap())
            .await
            .unwrap()
        {
            cog
        } else {
            return PathBuf::from(
                unsafe { COORDS_CONFIG.get() }
                    .unwrap()
                    .default_attachment_path
                    .as_str(),
            );
        };

        if let Some(subcog) = cog.subcategories.get(&subcog.to_string()) {
            PathBuf::from(
                subcog.attachment_path.as_ref().unwrap_or(
                    cog.attachment_path.as_ref().unwrap_or(
                        &unsafe { COORDS_CONFIG.get() }
                            .unwrap()
                            .default_attachment_path,
                    ),
                ),
            )
        } else {
            PathBuf::from(
                cog.attachment_path.as_ref().unwrap_or(
                    &unsafe { COORDS_CONFIG.get() }
                        .unwrap()
                        .default_attachment_path,
                ),
            )
        }
    }
}
