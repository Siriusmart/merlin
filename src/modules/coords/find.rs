use std::collections::HashMap;
use std::fmt::Write;

use mongodb::bson::Document;
use serenity::{
    all::{Context, Message, UserId},
    async_trait,
    futures::StreamExt,
};

use crate::sys::Command;

use super::{category::Category, collection::COORDS};

const PAGE_SIZE: usize = 5;

pub struct CmdFind;

#[async_trait]
impl Command for CmdFind {
    fn name(&self) -> &str {
        "find"
    }

    fn description(&self) -> &str {
        "Search for coord DB entries."
    }

    fn usage(&self) -> &[&str] {
        &["(name) (cog=value|page=value|near=x,z,radius)", "*"]
    }

    async fn run(&self, mut args: &[&str], ctx: &Context, msg: &Message) -> bool {
        let mut filter = Document::new();

        if let Some(name) = args.first() {
            if *name != "*" && !name.contains('=') {
                let name = name.replace(' ', "-").to_lowercase();
                filter.insert("name", name);
            }

            if !name.contains('=') {
                args = &args[1..]
            }
        } else {
            return false;
        }

        let mut page: Option<u32> = None;
        let mut near: Option<(i64, i64, i64)> = None;

        for arg in args.iter() {
            if let Some((left, right)) = arg.split_once('=') {
                match left {
                    "cog" => {
                        let (_cog, cog_id, subcog_id) =
                            if let Some(res) = Category::cogs_from_name(right).await {
                                res
                            } else {
                                let _ = msg.reply(ctx, "No maching results found.").await;
                                return true;
                            };

                        filter.insert("cog", cog_id);
                        if let Some(subcog) = subcog_id {
                            filter.insert("subcog", subcog);
                        }
                    }
                    "page" => {
                        if let Ok(parsed) = right.parse() {
                            page = Some(parsed);
                        } else {
                            let _ = msg.reply(ctx, "Could not parse page number.").await;
                            return true;
                        }
                    }
                    "near" => {
                        let args = right.splitn(3, ',').collect::<Vec<_>>();

                        if args.len() != 3 {
                            return false;
                        }

                        let x = args[0].parse::<i64>();
                        let z = args[1].parse::<i64>();
                        let r = args[2].parse::<u64>();

                        if x.is_err() || z.is_err() || r.is_err() {
                            let _ = msg.reply(ctx, "Could not parse nearbly arguments.").await;
                            return true;
                        }

                        near = Some((x.unwrap(), z.unwrap(), r.unwrap().pow(2) as i64))
                    }
                    _ => return false,
                }
            } else {
                return false;
            }
        }

        let mut cursor = unsafe { COORDS.get() }.unwrap().find(filter).await.unwrap();

        let to_skip = page.unwrap_or(0).saturating_sub(1) * PAGE_SIZE as u32;
        let mut skipped: u32 = 0;

        // allowed, display_name, name
        let mut clearance_lookup: HashMap<(i64, i64), (bool, String, String)> = HashMap::new();

        let mut entries = Vec::with_capacity(PAGE_SIZE);

        while let Some(entry) = cursor.next().await {
            let entry = entry.unwrap();

            if !entry.is_allowed(ctx, msg, &mut clearance_lookup).await {
                continue;
            }

            if let Some((x, z, r2)) = near {
                if (x - entry.x).pow(2) + z.pow(2) > r2 {
                    continue;
                }
            }

            if skipped < to_skip {
                skipped += 1;
                continue;
            }

            entries.push(entry);

            if entries.len() == PAGE_SIZE {
                break;
            }
        }

        match entries.len() {
            0 => {
                let _ = msg.reply(ctx, "No maching results found.").await;
            }
            1 => {
                let entry = &entries[0];
                let (_, display, name) = clearance_lookup.get(&(entry.cog, entry.subcog)).unwrap();
                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "**[{}{}] {}**\n{}\n\nx={} z={} in the {}{}",
                            display,
                            if display != name {
                                format!(" **({name})**")
                            } else {
                                String::new()
                            },
                            entry.display_name,
                            if entry.description.is_empty() {
                                "This entry has no description."
                            } else {
                                entry.description.as_str()
                            },
                            entry.x,
                            entry.z,
                            entry.dim.unwrap_or_default(),
                            if let Ok(user) = UserId::new(entry.author_id).to_user(ctx).await {
                                format!("\n\n*Entry added by {}.*", user.name)
                            } else {
                                String::new()
                            }
                        ),
                    )
                    .await;
            }
            _ => {
                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "Showing {} results.{}",
                            entries.len(),
                            entries.iter().zip(to_skip + 1..).fold(
                                String::new(),
                                |mut current, (entry, no)| {
                                    let (_, display, name) =
                                        clearance_lookup.get(&(entry.cog, entry.subcog)).unwrap();
                                    write!(
                                        current,
                                        "\n{no}. **{}**{} in {}{}",
                                        entry.display_name,
                                        if entry.display_name != entry.name {
                                            format!(" ({})", entry.name)
                                        } else {
                                            String::new()
                                        },
                                        display,
                                        if display != name {
                                            format!(" ({})", name)
                                        } else {
                                            String::new()
                                        }
                                    )
                                    .unwrap();
                                    current
                                }
                            )
                        ),
                    )
                    .await;
            }
        }

        true
    }
}
