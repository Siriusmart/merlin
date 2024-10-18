use std::collections::HashMap;
use std::fmt::Write;

use mongodb::bson::Document;
use serenity::{
    all::{Context, Message, UserId},
    async_trait,
    futures::StreamExt,
};

use crate::{sys::Command, Clearance, CollectionItem};

use super::{
    category::Category,
    collection::{CATEGORIES, COORDS},
    coord::Coord,
};

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
        &["(name) (cog=value|page=value)", "*"]
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

        async fn is_allowed(
            entry: &Coord,
            ctx: &Context,
            msg: &Message,
            lookup: &mut HashMap<(i64, i64), (bool, String, String)>,
        ) -> bool {
            if let Some((b, ..)) = lookup.get(&(entry.cog, entry.subcog)) {
                return *b;
            }

            async fn is_allowed_core(
                entry: &Coord,
                ctx: &Context,
                msg: &Message,
            ) -> (bool, String, String) {
                match (entry.cog, entry.subcog) {
                    (0, 0) => (
                        true,
                        "generic.unspecified".to_string(),
                        "generic.unspecified".to_string(),
                    ),
                    (0, 1) => (
                        entry.author_id == msg.author.id.get(),
                        "generic.private".to_string(),
                        "generic.private".to_string(),
                    ),
                    (cog, subcog) => {
                        let cog = if let Some(cog) =
                            Category::find_by_id(cog, unsafe { CATEGORIES.get() }.unwrap())
                                .await
                                .unwrap()
                        {
                            cog
                        } else {
                            return (false, String::new(), String::new());
                        };

                        let mut allowed = Clearance::is_allowed(&cog.allowed, ctx, msg)
                            .await
                            .unwrap_or(true);

                        let mut subcog_display = None;
                        let mut subcog_name = None;

                        if let Some(subcog) = cog.subcategories.get(&subcog.to_string()) {
                            allowed &= Clearance::is_allowed(&subcog.allowed, ctx, msg)
                                .await
                                .unwrap_or(true);
                            subcog_display = Some(subcog.display_name.as_str());
                            subcog_name = Some(subcog.name.as_str());
                        }

                        (
                            allowed,
                            if allowed {
                                format!(
                                    "{}.{}",
                                    cog.display_name,
                                    subcog_display.unwrap_or("unspecified")
                                )
                            } else {
                                String::new()
                            },
                            if allowed {
                                format!("{}.{}", cog.name, subcog_name.unwrap_or("unspecified"))
                            } else {
                                String::new()
                            },
                        )
                    }
                }
            }

            let (allowed, display, name) = is_allowed_core(entry, ctx, msg).await;
            lookup.insert((entry.cog, entry.subcog), (allowed, display, name));
            allowed
        }

        let mut entries = Vec::with_capacity(PAGE_SIZE);

        while let Some(entry) = cursor.next().await {
            let entry = entry.unwrap();

            if !is_allowed(&entry, ctx, msg, &mut clearance_lookup).await {
                continue;
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
