use std::sync::Arc;

use serenity::{all::ShardManager, prelude::TypeMapKey};
use tokio::time::Instant;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub struct StartInstanceContainer {
    start: Instant,
}

impl StartInstanceContainer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn get(&self) -> &Instant {
        &self.start
    }
}

impl TypeMapKey for StartInstanceContainer {
    type Value = Self;
}
