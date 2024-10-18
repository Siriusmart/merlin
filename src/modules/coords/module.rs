use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use serenity::async_trait;

use crate::{Command, Module, Mongo};

use super::{
    addcog::CmdAddcog,
    addcoord::CmdAddCoord,
    cog::CmdCog,
    collection::{CATEGORIES, COORDS},
    editcog::CmdEditCog,
    find::CmdFind,
};

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

        {
            let cmd: Box<dyn Command> = Box::new(CmdCog);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdEditCog);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdAddCoord);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdFind);
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

        if unsafe { COORDS.get() }.is_some() {
            unsafe { COORDS = OnceLock::new() };
        }

        let _ = unsafe { CATEGORIES.set(Mongo::database().collection("coords-cogs")) };
        let _ = unsafe { COORDS.set(Mongo::database().collection("coords-coords")) };
    }

    fn aliases(&self) -> &[(&str, &str)] {
        &[
            ("addcog", "coords addcog"),
            ("addcoord", "coords addcoord"),
            ("cog", "coords cog"),
            ("editcog", "coords editcog"),
            ("find", "coords find"),
        ]
    }
}
