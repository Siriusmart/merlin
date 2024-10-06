use std::{collections::HashMap, sync::Arc};

use serenity::{async_trait, Client};

use crate::{Command, Module};

use super::{
    keys::{ShardManagerContainer, StartInstanceContainer},
    ping::CmdPing,
    uptime::CmdUptime,
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

    async fn setup(&mut self, client: &Client) {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<StartInstanceContainer>(StartInstanceContainer::new());
    }

    fn aliases(&self) -> &[(&str, &str)] {
        &[("ping", "core ping"), ("uptime", "core uptime")]
    }
}
