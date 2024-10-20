use std::{collections::HashMap, hash::Hash, sync::OnceLock};

use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildId, Message, RoleId};

use super::Config;

static mut CLEARANCES: OnceLock<Clearance> = OnceLock::new();

#[derive(Serialize, Deserialize)]
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

    pub fn set(entry: String, list: &[&str]) -> bool {
        if list.iter().any(|line| line.starts_with('?')) {
            return false;
        }

        if !Self::validate(list, true) {
            return false;
        }

        let clearance = unsafe { CLEARANCES.get_mut() }.unwrap();
        clearance
            .0
            .insert(entry, list.iter().map(|s| s.to_string()).collect());

        true
    }

    pub fn validate(allowed_list: &[&str], presets: bool) -> bool {
        for entry in allowed_list {
            if entry.starts_with('?') {
                if !presets {
                    return false;
                }
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
                    if msg.author.id.get().to_string().as_str() == &entry[2..]
                        || msg.author.name.as_str() == &entry[2..]
                    {
                        return Some(entry_allowed);
                    }
                }
                '%' => {
                    if msg.guild_id.is_some_and(|guild| {
                        guild.get().to_string().as_str() == &entry[2..]
                            || guild.name(ctx).unwrap() == entry[2..]
                    }) {
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
                    let (guild, role) = entry[2..].split_once(':').unwrap_or(("", ""));
                    let guild = GuildId::new(guild.parse().unwrap_or_default());

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
