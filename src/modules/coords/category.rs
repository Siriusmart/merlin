use std::collections::HashMap;

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{modules::coords::collection::CATEGORIES, CollectionItem, Counter, Mongo};

#[derive(Serialize, Deserialize, Clone)]
pub struct Category {
    #[serde(rename = "_id")]
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub allowed: Vec<String>,
    pub subcategories: HashMap<String, Subcategory>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Subcategory {
    pub name: String,
    pub description: String,
    pub allowed: Vec<String>,
}

impl CollectionItem<i64> for Category {
    fn id(&self) -> i64 {
        self.id
    }
}

impl Subcategory {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            allowed: Default::default(),
        }
    }
}

impl Category {
    pub async fn get(display_name: &str) -> Option<Self> {
        let name = display_name.replace(' ', "-").to_lowercase();

        let categories = unsafe { CATEGORIES.get() }.unwrap();

        categories.find_one(doc! {"name": &name}).await.unwrap()
    }

    pub async fn new(display_name: String, description: String) -> Result<Self, &'static str> {
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
            return Err("a category with that name already exist");
        }

        let out = Category {
            id: Counter::bump_get("coords-categories", Mongo::database())
                .await
                .unwrap(),
            name,
            display_name,
            description,
            allowed: Vec::new(),
            subcategories: HashMap::new(),
        };

        out.save_create(categories).await.unwrap();

        Ok(out)
    }
}
