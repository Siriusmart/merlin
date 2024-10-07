use std::{collections::HashMap, sync::Arc};

use serenity::async_trait;

use crate::{Command, Module};

pub struct ModCore(Arc<HashMap<String, Box<dyn Command>>>);

impl Default for ModCore {
    fn default() -> Self {
        Self::new()
    }
}

impl ModCore {
    pub fn new() -> Self {
        let map = HashMap::new();

        Self(Arc::new(map))
    }
}

#[async_trait]
impl Module for ModCore {
    fn name(&self) -> &str {
        "core"
    }

    fn description(&self) -> &str {
        "Core service modules."
    }

    fn commands(&self) -> Arc<HashMap<String, Box<dyn Command>>> {
        self.0.clone()
    }
}
