use std::collections::HashMap;

use mongodb::bson::{doc, Document};
use serenity::{
    all::{Context, Message},
    async_trait,
    futures::StreamExt,
};

use crate::{sys::Command, PerCommandConfig};

use super::{category::Category, collection::COORDS};

pub struct CmdCoordRm;

#[async_trait]
impl Command for CmdCoordRm {
    fn name(&self) -> &str {
        "coordrm"
    }

    fn description(&self) -> &str {
        "Remove coord DB entries."
    }

    fn usage(&self) -> &[&str] {
        &[
            "(name|regex|id|*) (cog=value|page=value|desc=regex|near=x,z,radius|dim=ow/nether/end|tags=tag1,tag2..)",
            "(category) (page=value|desc=regex|near=x,z,radius|dim=ow/nether/end|tags=tag1,tag2..)"
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

        for entry in entries.iter() {
            let _ = unsafe { COORDS.get() }
                .unwrap()
                .delete_one(doc! { "_id": entry.id })
                .await
                .unwrap();
        }

        let _ = msg
            .reply(
                ctx,
                format!(
                    "{} {} removed.",
                    entries.len(),
                    if entries.len() > 1 {
                        "entries"
                    } else {
                        "entry"
                    }
                ),
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
