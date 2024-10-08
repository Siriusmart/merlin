use std::sync::OnceLock;

use mongodb::{bson::Document, options::ClientOptions, Client, Collection, Database};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::Config;

use super::{
    collections::{COUNTERS_DESER, COUNTERS_SER, DATABASE},
    Counter,
};

pub struct Mongo;

impl Mongo {
    pub async fn load() {
        let config = MongoConfig::load();

        let client = Client::with_options(
            ClientOptions::parse(&config.address)
                .await
                .unwrap_or_else(|_| panic!("could not resolve mongodb host {}", config.address)),
        )
        .unwrap();

        let db = client.database(&config.db);

        let counters_deser: Collection<Counter> = db.collection(&config.counters);
        let counters_ser: Collection<Document> = db.collection(&config.counters);

        unsafe { DATABASE.set(db) }.unwrap();
        unsafe { COUNTERS_DESER.set(counters_deser) }.unwrap();
        unsafe { COUNTERS_SER.set(counters_ser) }.unwrap();
    }

    pub async fn reload() {
        unsafe { DATABASE = OnceLock::new() };
        unsafe { COUNTERS_DESER = OnceLock::new() };
        unsafe { COUNTERS_SER = OnceLock::new() };

        Self::load().await;
    }

    pub fn database() -> &'static Database {
        unsafe { DATABASE.get() }.unwrap()
    }
}

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Hash)]
struct MongoConfig {
    #[serde_inline_default("mongodb://localhost:27017".to_string())]
    pub address: String,
    #[serde_inline_default("merlin".to_string())]
    pub db: String,
    #[serde_inline_default("counters".to_string())]
    pub counters: String,
}

impl Config for MongoConfig {
    const NAME: &'static str = "mongodb";
    const NOTE: &'static str = "MongoDB options";
}
