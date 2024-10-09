use std::{collections::HashMap, sync::Arc};

use serenity::async_trait;

use crate::{Command, CommandHandler, Module};

use super::{
    clearance::CmdClearance,
    keys::{ShardManagerContainer, StartInstanceContainer},
    ping::CmdPing,
    reload::CmdReload,
    save::CmdSave,
    switch::CmdSwitch,
    uptime::CmdUptime,
    version::CmdVersion,
};

pub struct ModCore(Arc<HashMap<String, Box<dyn Command>>>);

impl Default for ModCore {
    fn default() -> Self {
        Self::new()
    }
}

impl ModCore {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        {
            let cmd: Box<dyn Command> = Box::new(CmdPing);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdUptime);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdVersion);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdReload);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdSwitch);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdSave);
            map.insert(cmd.name().to_string(), cmd);
        }

        {
            let cmd: Box<dyn Command> = Box::new(CmdClearance);
            map.insert(cmd.name().to_string(), cmd);
        }

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

    async fn setup(&mut self) {
        let client = CommandHandler::client_mut();
        let mut data = client.data.write().await;
        data.entry::<ShardManagerContainer>()
            .or_insert(client.shard_manager.clone());
        data.entry::<StartInstanceContainer>()
            .or_insert(StartInstanceContainer::new());
    }

    fn aliases(&self) -> &[(&str, &str)] {
        &[
            ("ping", "core ping"),
            ("uptime", "core uptime"),
            ("version", "core version"),
            ("reload", "core reload"),
            ("switch", "core switch"),
            ("save", "core save"),
            ("clearance", "core clearance"),
            ("preset", "core clearance"),
        ]
    }
}
