use std::sync::OnceLock;

use mongodb::Collection;

use super::category::Category;

pub static mut CATEGORIES: OnceLock<Collection<Category>> = OnceLock::new();
