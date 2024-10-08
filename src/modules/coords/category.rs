use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{modules::coords::collection::CATEGORIES, CollectionItem, Counter, Mongo};

#[derive(Serialize, Deserialize, Clone)]
pub struct Category {
    #[serde(rename = "_id")]
    id: i64,
    name: String,
    display_name: String,
    description: String,
    allowed: Vec<String>,
    subcategories: Vec<SubCategory>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubCategory {
    id: i64,
    name: String,
    description: String,
    allowed: Vec<String>,
}

impl CollectionItem<i64> for Category {
    fn id(&self) -> i64 {
        self.id
    }
}

impl Category {
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
            subcategories: Vec::new(),
        };

        out.save_create(categories).await.unwrap();

        Ok(out)
    }
}
