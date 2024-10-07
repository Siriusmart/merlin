use serde::{Deserialize, Serialize};

use crate::CollectionItem;

#[derive(Serialize, Deserialize, Clone)]
pub struct Category {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    description: String,
    allowed: Vec<String>,
    subcategories: Vec<SubCategory>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubCategory {
    name: String,
    description: String,
    allowed: Vec<String>,
}

impl CollectionItem<String> for Category {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Category {
    pub fn new(name: String) -> Result<Self, &'static str> {
        let id = name.replace(' ', "_").to_lowercase();

        if id.chars().any(|c| !c.is_alphanumeric() && c != '_') {
            return Err("contains non ascii character");
        }

        // Category::find_by_id(id, )

        todo!()
    }
}
