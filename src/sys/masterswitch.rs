use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use serenity::all::{Context, Message, RoleId};

use super::Config;

static SWITCH: OnceLock<MasterSwitch> = OnceLock::new();

#[derive(Serialize, Deserialize, Default)]
pub struct MasterSwitch(pub HashMap<String, PerModuleConfig>);

impl MasterSwitch {
    pub fn get(module: &str) -> &PerModuleConfig {
        SWITCH.get().unwrap().0.get(module).unwrap()
    }
}

impl Config for MasterSwitch {
    const NAME: &'static str = "switch";
    const NOTE: &'static str = "Master switch for each module";
}

impl MasterSwitch {
    pub fn finalise(self) {
        let _ = SWITCH.set(self);
    }
}

#[derive(Serialize, Deserialize)]
pub struct PerModuleConfig {
    pub enabled: bool,
    pub allowed: Vec<String>,
    pub commands: HashMap<String, PerCommandConfig>,
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

#[derive(Serialize, Deserialize)]
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
    pub async fn is_allowed(&self, ctx: &Context, msg: &Message) -> Option<bool> {
        is_allowed(&self.allowed, ctx, msg)
            .await
            .map(|b| self.enabled && b)
    }
}

impl PerCommandConfig {
    pub async fn is_allowed(&self, ctx: &Context, msg: &Message) -> Option<bool> {
        is_allowed(&self.allowed, ctx, msg)
            .await
            .map(|b| self.enabled && b)
    }
}

async fn is_allowed(allowed_list: &[String], ctx: &Context, msg: &Message) -> Option<bool> {
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
