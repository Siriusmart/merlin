use std::sync::OnceLock;

use mongodb::{bson::Document, Collection, Database};

use super::Counter;

pub static mut DATABASE: OnceLock<Database> = OnceLock::new();

pub static mut COUNTERS_SER: OnceLock<Collection<Document>> = OnceLock::new();
pub static mut COUNTERS_DESER: OnceLock<Collection<Counter>> = OnceLock::new();
