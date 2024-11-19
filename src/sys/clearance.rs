use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::OnceLock,
};

use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildId, Message, RoleId};

use super::Config;

static mut CLEARANCES: OnceLock<Clearance> = OnceLock::new();

#[derive(Serialize, Deserialize, Clone)]
pub struct Clearance(pub HashMap<String, Vec<String>>);

impl Hash for Clearance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut clearances = self.0.iter().collect::<Vec<_>>();
        clearances.sort_by_key(|entry| entry.0);
        clearances.hash(state);
    }
}

impl Default for Clearance {
    fn default() -> Self {
        Self(HashMap::from([(
            "admin".to_string(),
            vec!["-everyone".to_string()],
        )]))
    }
}

impl Config for Clearance {
    const NAME: &'static str = "clearance";
    const NOTE: &'static str = "Clearance level presets";
}

impl Clearance {
    pub fn no_cycles(&self, name: &str, set: &mut HashSet<String>) -> bool {
        if !set.insert(name.to_string()) {
            return false;
        }

        let rules = if let Some(rules) = self.0.get(name) {
            rules
        } else {
            return true;
        };

        for entry in rules.iter() {
            if entry.starts_with('?') && !self.no_cycles(&entry[1..], set) {
                return false;
            }
        }

        true
    }

    pub fn remove(entry: &str) -> bool {
        unsafe { CLEARANCES.get_mut() }
            .unwrap()
            .0
            .remove(entry)
            .is_some()
    }

    pub fn write_to_config() {
        unsafe { CLEARANCES.get() }.unwrap().smart_save();
    }

    pub fn list_all() -> Vec<&'static String> {
        unsafe { CLEARANCES.get() }.unwrap().0.keys().collect()
    }

    pub fn setup() {
        let _ = unsafe { CLEARANCES.set(Clearance::load()) };
    }

    pub fn reload() {
        unsafe { CLEARANCES = OnceLock::new() };
        let _ = unsafe { CLEARANCES.set(Clearance::load()) };
    }

    pub async fn map_rules(list: &mut [String], msg: &Message, ctx: &Context) -> bool {
        for rule in list.iter_mut() {
            if rule.chars().nth(1) == Some('&') && !rule.contains(':') {
                if msg.guild_id.is_none() {
                    let _ = msg
                        .reply(ctx, "Server ID of role based rules cannot be inferred.")
                        .await;
                    return false;
                }

                *rule = format!(
                    "{}{}:{}",
                    &rule[0..2],
                    msg.guild_id.unwrap().get(),
                    &rule[2..]
                );
            }
        }

        true
    }

    pub fn set(entry: String, list: &[&str]) -> bool {
        if !Self::validate(list, Some(&entry)) {
            return false;
        }

        let clearance = unsafe { CLEARANCES.get_mut() }.unwrap();
        clearance
            .0
            .insert(entry, list.iter().map(|s| s.to_string()).collect());

        true
    }

    pub fn validate(allowed_list: &[&str], preset: Option<&str>) -> bool {
        let mut used_presets = Vec::new();

        for entry in allowed_list {
            if let Some(preset) = entry.strip_prefix('?') {
                used_presets.push(preset);
                continue;
            }

            if entry.len() < 2 || !matches!(entry.chars().next().unwrap(), '+' | '-') {
                return false;
            }

            match entry.chars().nth(1).unwrap() {
                '@' | '%' | '#' | '&' if entry.len() > 2 => {}
                _ if matches!(&entry[1..], "everyone" | "everywhere" | "dm" | "server") => {}
                _ => return false,
            }
        }

        if preset.is_none() {
            return true;
        }

        let mut new_clearance = unsafe { CLEARANCES.get() }.unwrap().clone();
        new_clearance.0.insert(
            preset.unwrap().to_string(),
            allowed_list.iter().map(|s| s.to_string()).collect(),
        );

        for preset in used_presets.iter() {
            if !new_clearance.no_cycles(preset, &mut HashSet::new()) {
                return false;
            }
        }

        true
    }

    pub fn get(level: &str) -> &[String] {
        unsafe { CLEARANCES.get() }
            .unwrap()
            .0
            .get(level)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    #[async_recursion]
    pub async fn eval(level: &str, ctx: &Context, msg: &Message) -> Option<bool> {
        match unsafe { CLEARANCES.get() }.unwrap().0.get(level) {
            Some(list) => Self::is_allowed(list, ctx, msg).await,
            None => None,
        }
    }

    pub async fn is_allowed(allowed_list: &[String], ctx: &Context, msg: &Message) -> Option<bool> {
        for entry in allowed_list.iter().rev() {
            let entry_allowed = match entry.chars().next().unwrap() {
                '+' => true,
                '-' => false,
                '?' => {
                    if let Some(b) = Self::eval(&entry[1..], ctx, msg).await {
                        return Some(b);
                    }
                    continue;
                }
                c => panic!("unknown per command entry modifier {c}"),
            };

            match entry.chars().nth(1).unwrap() {
                '@' => {
                    if let Ok(id) = entry[2..].parse::<u64>() {
                        if id == msg.author.id.get() {
                            return Some(entry_allowed);
                        }
                    } else if msg.author.name.as_str() == &entry[2..] {
                        return Some(entry_allowed);
                    }
                }
                '%' => {
                    if let Some(guild) = msg.guild_id {
                        if let Ok(id) = entry[2..].parse::<u64>() {
                            if id == guild.get() {
                                return Some(entry_allowed);
                            }
                        } else if guild.name(ctx).unwrap() == entry[2..] {
                            return Some(entry_allowed);
                        }
                    }
                }
                '#' => {
                    if let Ok(id) = entry[2..].parse::<u64>() {
                        if id == msg.channel_id.get() {
                            return Some(entry_allowed);
                        }
                    } else if msg.guild_id.is_some()
                        && msg.channel_id.name(ctx).await.unwrap_or_default() == entry[2..]
                    {
                        return Some(entry_allowed);
                    }
                }
                '&' => {
                    let (guild, role) = entry[2..].split_once(':').unwrap_or(("", ""));
                    let guild = GuildId::new(guild.parse().unwrap_or(1));

                    if msg
                        .author
                        .has_role(
                            ctx,
                            guild,
                            match role.parse() {
                                Ok(id) => RoleId::new(id),
                                Err(_) => guild
                                    .roles(ctx)
                                    .await
                                    .unwrap_or_default()
                                    .values()
                                    .find(|role| role.name.as_str() == &entry[2..])
                                    .map(|role| role.id)
                                    .unwrap_or_else(|| RoleId::new(1)),
                            },
                        )
                        .await
                        .unwrap_or(false)
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
}
