use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use serenity::async_trait;

use crate::{Command, Module, Mongo};

use super::{
    attach::CmdAttach,
    cog::CmdCog,
    cogadd::CmdCogAdd,
    cogedit::CmdCogEdit,
    cogperms::CmdCogPerms,
    cogrm::CmdCogRm,
    collection::{CATEGORIES, COORDS},
    config::CoordsConfig,
    coordadd::CmdCoordAdd,
    coordedit::CmdCoordEdit,
    coordrm::CmdCoordRm,
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
            let cmd: Box<dyn Command> = Box::new(CmdCogAdd);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdCog);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdCogEdit);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdCoordAdd);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdFind);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdCogPerms);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdCoordEdit);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdCoordRm);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdCogRm);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdAttach);
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
        CoordsConfig::setup();
        let _ = unsafe { CATEGORIES.set(Mongo::database().collection("coords-cogs")) };
        let _ = unsafe { COORDS.set(Mongo::database().collection("coords-coords")) };
    }

    async fn reload(&mut self) {
        CoordsConfig::reload();
        unsafe { CATEGORIES = OnceLock::new() };
        unsafe { COORDS = OnceLock::new() };

        let _ = unsafe { CATEGORIES.set(Mongo::database().collection("coords-cogs")) };
        let _ = unsafe { COORDS.set(Mongo::database().collection("coords-coords")) };
    }

    fn aliases(&self) -> &[(&str, &str)] {
        &[
            ("cogadd", "coords cogadd"),
            ("coordadd", "coords coordadd"),
            ("cog", "coords cog"),
            ("cogedit", "coords cogedit"),
            ("cogperms", "coords cogperms"),
            ("coordedit", "coords coordedit"),
            ("find", "coords find"),
            ("coordrm", "coords coordrm"),
            ("cogrm", "coords cogrm"),
            ("attach", "coords attach"),
        ]
    }
}
