use std::collections::HashMap;
use std::fmt::Write;

use mongodb::bson::{doc, Document};
use serenity::{
    all::{Context, Message, UserId},
    async_trait,
    futures::StreamExt,
};

use crate::{sys::Command, PerCommandConfig};

use super::{category::Category, collection::COORDS, config::COORDS_CONFIG};

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
        &[
            "(name) (cog=value|page=value|near=x,z,radius|dim=ow/nether/end|tags=tag1,tag2..)",
            "(category) (page=value|near=x,z,radius|dim=ow/nether/end|tags=tag1,tag2..)",
            "*",
        ]
    }

    async fn run(&self, mut args: &[&str], ctx: &Context, msg: &Message) -> bool {
        let mut filter = Document::new();
        let mut name = None;

        if let Some(first) = args.first() {
            if let Some((_cog, cog_id, subcog_id)) = Category::cogs_from_name(first).await {
                filter.insert("cog", cog_id);
                if let Some(subcog) = subcog_id {
                    filter.insert("subcog", subcog);
                }
            } else if *first != "*" && !first.contains('=') {
                let formatted_name = first.replace(' ', "-").to_lowercase();
                filter.insert("name", doc! { "$regex": &formatted_name });
                name = Some(formatted_name);
            }

            if !first.contains('=') {
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
                            let _ = msg.reply(ctx, "Could not parse nearby arguments.").await;
                            return true;
                        }

                        near = Some((x.unwrap(), z.unwrap(), r.unwrap().pow(2) as i64))
                    }
                    "dim" if matches!(right, "ow" | "nether" | "end") => {
                        filter.insert("dim", right).unwrap();
                    }
                    "tags" => {
                        filter.insert("tags", doc! { "$all": right.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>()});
                    }
                    _ => return false,
                }
            } else {
                return false;
            }
        }

        if filter.contains_key("near") && !filter.contains_key("dim") {
            let _ = msg
                .reply(ctx, "Nearby search requires dimension to be specified.")
                .await;
            return true;
        }

        let mut cursor = unsafe { COORDS.get() }
            .unwrap()
            .find(filter)
            .sort(doc! {"$natural": -1})
            .await
            .unwrap();

        let page_size = unsafe { COORDS_CONFIG.get() }.unwrap().page_size;
        let to_skip = page.unwrap_or(0).saturating_sub(1) * page_size;

        // allowed, display_name, name
        let mut clearance_lookup: HashMap<(i64, i64), (bool, String, String)> = HashMap::new();

        let mut entries_owned =
            Vec::with_capacity(unsafe { COORDS_CONFIG.get() }.unwrap().page_size as usize);

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

            if Some(&entry.name) == name.as_ref() {
                entries_owned = vec![entry];
                break;
            }

            entries_owned.push(entry);
        }

        let entries = if entries_owned.len() > page_size as usize {
            if to_skip + page_size > entries_owned.len() as u32 {
                &entries_owned[entries_owned.len() - page_size as usize..]
            } else {
                &entries_owned[to_skip as usize..(to_skip + page_size) as usize]
            }
        } else {
            &entries_owned
        };

        let has_next_page = cursor.next().await.is_some();

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
                            "**[{}{}] {}**\n{}\n{}\n\nx={} z={} in the {}{}",
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
                            if entry.tags.is_empty() {
                                "This entry isn't tagged.".to_string()
                            } else {
                                format!("Tags: {}", entry.tags.join(", "))
                            },
                            entry.x,
                            entry.z,
                            entry.dim,
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
                            "Showing {} results.{}{}",
                            entries.len(),
                            entries.iter().zip(to_skip + 1..).fold(
                                String::new(),
                                |mut current, (entry, no)| {
                                    let (_, display, name) =
                                        clearance_lookup.get(&(entry.cog, entry.subcog)).unwrap();
                                    write!(
                                        current,
                                        "\n{no}. **{}**{} in {}{}{}",
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
                                        },
                                        if entry.tags.is_empty() {
                                            String::new()
                                        } else {
                                            format!(" (tags: {})", entry.tags.join(", "))
                                        }
                                    )
                                    .unwrap();
                                    current
                                }
                            ),
                            if has_next_page {
                                "\n*(continued next page)*"
                            } else {
                                "\n*(there are no more results)*"
                            }
                        ),
                    )
                    .await;
            }
        }

        true
    }

    fn percmd(&self) -> PerCommandConfig {
        PerCommandConfig {
            allowed: vec!["-everyone".to_string(), "?coorduser".to_string()],
            ..Default::default()
        }
    }
}
