use std::collections::HashMap;

use chrono::{Datelike, Timelike};
use mongodb::bson::{doc, Document};
use serenity::{
    all::{Context, CreateAllowedMentions, EditMessage, Message},
    async_trait,
    futures::StreamExt,
};
use tokio::{fs, io::AsyncWriteExt};

use crate::{sys::Command, PerCommandConfig};

use super::{category::Category, collection::COORDS};

pub struct CmdAttach;

#[async_trait]
impl Command for CmdAttach {
    fn name(&self) -> &str {
        "attach"
    }

    fn description(&self) -> &str {
        "Attach files to a coord entry."
    }

    fn usage(&self) -> &[&str] {
        &[
            "(name|regex|id|*) (cog=value|page=value|desc=regex|near=x,z,radius|dim=ow/nether/end|tags=tag1,tag2..) [attachments]",
            "(category) (page=value|desc=regex|near=x,z,radius|dim=ow/nether/end|tags=tag1,tag2..) [attachments]"
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
                if let Ok(id) = first.parse::<i64>() {
                    filter.insert("_id", id);
                } else {
                    let formatted_name = first.replace(' ', "-").to_lowercase();
                    filter.insert("name", doc! { "$regex": &formatted_name });
                    name = Some(formatted_name);
                }
            }

            if !first.contains('=') {
                args = &args[1..]
            }
        } else {
            return false;
        }

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
                    "desc" => {
                        filter.insert("description", doc! { "$regex": right });
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

        let mut cursor = unsafe { COORDS.get() }.unwrap().find(filter).await.unwrap();

        // allowed, display_name, name
        let mut clearance_lookup: HashMap<(i64, i64), (bool, String, String)> = HashMap::new();

        let mut entries = Vec::new();

        while let Some(entry) = cursor.next().await {
            let entry = entry.unwrap();

            if !entry.is_allowed(ctx, msg, &mut clearance_lookup).await {
                continue;
            }

            if let Some((x, z, r2)) = near {
                if (x - entry.x).pow(2) + (z - entry.z).pow(2) > r2 {
                    continue;
                }
            }

            if Some(&entry.name) == name.as_ref() {
                entries = vec![entry];
                break;
            }

            entries.push(entry);
        }

        match entries.len() {
            0 => {
                let _ = msg.reply(ctx, "No entries found.").await;
                return true;
            }
            1 => {}
            c => {
                let _ = msg.reply(ctx, format!("You can only attach a file to a single entry, but there are {c} matching entries.")).await;
                return true;
            }
        }

        let entry = entries.first().unwrap();

        if (entry.cog, entry.subcog) == (0, 1) {
            let _ = msg
                .reply(
                    ctx,
                    "You cannot attach file to entries in **generic.private**.",
                )
                .await;
            return true;
        }

        if msg.attachments.is_empty() {
            return false;
        }

        let dir_path = Category::path(entry.cog, entry.subcog)
            .await
            .join(entry.id.to_string());

        let mut replied = msg.reply(ctx, "Upload has started.").await.unwrap();

        if !fs::try_exists(&dir_path).await.unwrap() {
            fs::create_dir_all(&dir_path).await.unwrap();
        }

        for attachment in msg.attachments.iter() {
            let content = match attachment.download().await {
                Ok(c) => c,
                Err(_) => {
                    let _ = replied
                        .edit(ctx, EditMessage::new().content("Upload failed."))
                        .await;
                    return true;
                }
            };

            let username = &msg.author.name;
            let label = &attachment.filename;
            let id = attachment.id.get();
            let year = msg.timestamp.year();
            let month = msg.timestamp.month();
            let day = msg.timestamp.day();
            let hour = msg.timestamp.hour();
            let min = msg.timestamp.minute();
            let sec = msg.timestamp.second();

            let mut file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(dir_path.join(format!("{year}{month:0>2}{day:0>2}-{hour:0>2}{min:0>2}{sec:0>2}_{username}_{id}_{label}"))).await.unwrap();

            file.write_all(&content).await.unwrap();
        }

        let _ = replied
            .edit(
                ctx,
                EditMessage::new()
                    .content("Upload completed.")
                    .allowed_mentions(CreateAllowedMentions::new().all_users(false)),
            )
            .await;

        true
    }

    fn percmd(&self) -> PerCommandConfig {
        PerCommandConfig {
            allowed: vec!["-everyone".to_string(), "?coorduser".to_string()],
            ..Default::default()
        }
    }
}
