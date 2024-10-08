use std::{collections::HashMap, hash::Hash, sync::OnceLock};

use serde::{Deserialize, Serialize};
use serenity::all::{Context, Message, RoleId};

use super::Config;

static mut SWITCH: OnceLock<MasterSwitch> = OnceLock::new();

#[derive(Serialize, Deserialize, Default)]
pub struct MasterSwitch(pub HashMap<String, PerModuleConfig>);

impl Hash for MasterSwitch {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut modules = self.0.iter().collect::<Vec<_>>();
        modules.sort_by_key(|entry| entry.0);
        modules.hash(state);
    }
}

impl MasterSwitch {
    pub fn get(module: &str) -> Option<&PerModuleConfig> {
        unsafe { SWITCH.get() }.unwrap().0.get(module)
    }

    pub fn finalise(self) {
        let _ = unsafe { SWITCH.set(self) };
    }

    pub fn write_to_config() {
        unsafe { SWITCH.get() }.unwrap().smart_save();
    }

    pub fn reload() -> &'static mut Self {
        let new = Self::load();
        unsafe { SWITCH = OnceLock::new() };
        let _ = unsafe { SWITCH.set(new) };
        unsafe { SWITCH.get_mut().unwrap() }
    }

    pub fn switch(module: &str, command: Option<&str>, value: bool) -> bool {
        let switch = unsafe { SWITCH.get_mut() }.unwrap();
        let permod = match switch.0.get_mut(module) {
            Some(module) => module,
            None => return false,
        };

        let cmd = match command {
            Some(cmd) => cmd,
            None => {
                permod.enabled = value;
                return true;
            }
        };

        let command = match permod.commands.get_mut(cmd) {
            Some(cmd) => cmd,
            None => return false,
        };

        command.enabled = value;

        true
    }
}

impl Config for MasterSwitch {
    const NAME: &'static str = "switch";
    const NOTE: &'static str = "Master switch for each module";
}

#[derive(Serialize, Deserialize)]
pub struct PerModuleConfig {
    pub enabled: bool,
    pub allowed: Vec<String>,
    pub commands: HashMap<String, PerCommandConfig>,
}

impl Hash for PerModuleConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.enabled.hash(state);
        self.allowed.hash(state);
        let mut command = self.commands.iter().collect::<Vec<_>>();
        command.sort_by_key(|entry| entry.0);
        command.hash(state);
    }
}

impl Default for PerModuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed: [].into_iter().map(str::to_string).collect(),
            commands: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Hash)]
pub struct PerCommandConfig {
    pub enabled: bool,
    pub allowed: Vec<String>,
}

impl Default for PerCommandConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed: Vec::new(),
        }
    }
}

impl PerModuleConfig {
    pub async fn is_allowed(&self, ctx: &Context, msg: &Message) -> bool {
        is_allowed(&self.allowed, ctx, msg)
            .await
            .map(|b| self.enabled && b)
            .unwrap_or(self.enabled)
    }
}

impl PerCommandConfig {
    pub async fn is_allowed(&self, ctx: &Context, msg: &Message) -> bool {
        is_allowed(&self.allowed, ctx, msg)
            .await
            .map(|b| self.enabled && b)
            .unwrap_or(self.enabled)
    }
}

pub async fn is_allowed(allowed_list: &[String], ctx: &Context, msg: &Message) -> Option<bool> {
    for entry in allowed_list.iter().rev() {
        let entry_allowed = match entry.chars().next().unwrap() {
            '+' => true,
            '-' => false,
            c => panic!("unknown per command entry modifier {c}"),
        };

        match entry.chars().nth(1).unwrap() {
            '@' => {
                if msg.author.id.get().to_string().as_str() == &entry[2..]
                    || msg.author.name.as_str() == &entry[2..]
                {
                    return Some(entry_allowed);
                }
            }
            '#' => {
                if msg.channel_id.get().to_string().as_str() == &entry[2..]
                    || msg.channel_id.name(ctx).await.unwrap_or_default() == entry[2..]
                {
                    return Some(entry_allowed);
                }
            }
            '&' => {
                if msg.guild_id.is_some()
                    && msg
                        .author
                        .has_role(
                            ctx,
                            msg.guild_id.unwrap(),
                            match entry[2..].parse() {
                                Ok(id) => RoleId::new(id),
                                Err(_) => msg
                                    .guild_id
                                    .unwrap()
                                    .roles(ctx)
                                    .await
                                    .unwrap()
                                    .values()
                                    .find(|role| role.name.as_str() == &entry[2..])
                                    .map(|role| role.id)
                                    .unwrap_or_else(|| RoleId::new(1)),
                            },
                        )
                        .await
                        .unwrap()
                {
                    return Some(entry_allowed);
                }
            }
            _ => match &entry[1..] {
                "everyone" | "everywhere" => {
                    return Some(entry_allowed);
                }
                "dm" => {
                    if msg.guild_id.is_none() {
                        return Some(entry_allowed);
                    }
                }
                "server" => {
                    if msg.guild_id.is_some() {
                        return Some(entry_allowed);
                    }
                }
                s => panic!("unknown per command entry scope {s}"),
            },
        }
    }

    None
}