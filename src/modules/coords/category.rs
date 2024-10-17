use std::collections::HashMap;

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{modules::coords::collection::CATEGORIES, CollectionItem, Counter, Mongo};

// special categories
// X.0 - uncategorised
// 0.1 - private: author only

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Category {
    #[serde(rename = "_id")]
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub allowed: Vec<String>,
    pub subcogcounter: i64,
    pub subcategories: HashMap<String, Subcategory>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Subcategory {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub allowed: Vec<String>,
}

impl CollectionItem<i64> for Category {
    fn id(&self) -> i64 {
        self.id
    }
}

impl Subcategory {
    pub fn new(name: String, display_name: String, description: String, id: i64) -> Self {
        Self {
            id,
            name,
            display_name,
            description,
            allowed: Default::default(),
        }
    }
}

impl Category {
    // category, category id, subcog id
    pub async fn cogs_from_name(name: &str) -> Option<(Option<Category>, i64, Option<i64>)> {
        if name == "generic.unspecified" {
            return Some((None, 0, Some(0)));
        }

        if let Some((left, right)) = name.split_once('.') {
            Self::get(left).await.and_then(|cog| {
                let id = cog.id;
                let subcog = cog.get_subcog(right)?.id;
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
            return Err("a category with that name already exists");
        }

        let out = Category {
            id: Counter::bump_get("coords-categories", Mongo::database())
                .await
                .unwrap(),
            name,
            display_name,
            description,
            allowed: Vec::new(),
            subcogcounter: 1,
            subcategories: HashMap::new(),
        };

        out.save_create(categories).await.unwrap();

        Ok(out)
    }
}
