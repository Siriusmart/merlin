use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use serenity::async_trait;

use crate::{Command, Module, Mongo};

use super::{addcog::CmdAddcog, collection::CATEGORIES};

pub struct ModCoords(Arc<HashMap<String, Box<dyn Command>>>);

impl Default for ModCoords {
    fn default() -> Self {
        Self::new()
    }
}

impl ModCoords {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        {
            let cmd: Box<dyn Command> = Box::new(CmdAddcog);
            map.insert(cmd.name().to_string(), cmd);
        }

        Self(Arc::new(map))
    }
}

#[async_trait]
impl Module for ModCoords {
    fn name(&self) -> &str {
        "coords"
    }

    fn description(&self) -> &str {
        "Coordinates DB for anarchy servers."
    }

    fn commands(&self) -> Arc<HashMap<String, Box<dyn Command>>> {
        self.0.clone()
    }

    async fn setup(&mut self) {
        if unsafe { CATEGORIES.get() }.is_some() {
            unsafe { CATEGORIES = OnceLock::new() };
        }

        let _ = unsafe { CATEGORIES.set(Mongo::database().collection("coords")) };
    }
}
