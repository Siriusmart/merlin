use std::sync::OnceLock;

use mongodb::Collection;

use super::{category::Category, coord::Coord};

pub static mut CATEGORIES: OnceLock<Collection<Category>> = OnceLock::new();
pub static mut COORDS: OnceLock<Collection<Coord>> = OnceLock::new();
