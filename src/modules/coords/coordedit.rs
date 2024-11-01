use std::collections::HashMap;

use mongodb::bson::{doc, Document};
use serenity::{
    all::{Context, Message},
    async_trait,
    futures::StreamExt,
};
use tokio::fs;

use crate::{sys::Command, Clearance, CollectionItem, PerCommandConfig};

use super::{
    category::Category,
    collection::COORDS,
    config::COORDS_CONFIG,
    coord::{Coord, Dimension},
};

pub struct CmdCoordEdit;

#[async_trait]
impl Command for CmdCoordEdit {
    fn name(&self) -> &str {
        "coordedit"
    }

    fn description(&self) -> &str {
        "Edit coord DB entries."
    }

    fn usage(&self) -> &[&str] {
        &[
            "(name|regex|id|*) (cog=value|desc=regex|near=x,z,radius|dim=ow/nether/end...|tags=tag1,tag2...) (newname=value|newdesc=value|newcog=value|newpos=x,z|newdim=ow/nether/end...|newtags=tag1,tag2...)",
            "(category) (desc=regex|near=x,z,radius|dim=ow/nether/end...|tags=tag1,tag2...) (newname=value|newdesc=value|newcog=value|newpos=x,z|newdim=ow/nether/end...|newtags=tag1,tag2...)",
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

        let mut newdisplay = None;
        let mut newdesc = None;
        let mut newcog = None;
        let mut newpos = None;
        let mut newdim = None;
        let mut newtags = None;

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
                    "newname" => newdisplay = Some(right),
                    "newdesc" => newdesc = Some(right),
                    "newcog" => newcog = Some(right),
                    "newpos" => newpos = Some(right),
                    "newdim" if matches!(right, "ow" | "nether" | "end") => {
                        newdim = Some(Dimension::from_str(right).unwrap())
                    }
                    "newtags" if right.is_empty() => newtags = Some(Vec::new()),
                    "newtags" => {
                        newtags = Some(
                            right
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<_>>(),
                        )
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

        if newpos.is_some() && newdim.is_none() {
            let _ = msg
                .reply(
                    ctx,
                    "Updating position requires new dimension to be specified.",
                )
                .await;
            return true;
        }

        let mut cursor = unsafe { COORDS.get() }.unwrap().find(filter).await.unwrap();

        // allowed, display_name, name
        let mut clearance_lookup: HashMap<(i64, i64), (bool, String, String)> = HashMap::new();

        let newcog_lower = newcog.map(|s| s.to_lowercase());

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

            let path = if let Some(cog) = &newcog_lower {
                let path = Category::path(entry.cog, entry.subcog)
                    .await
                    .join(entry.id.to_string());
                if fs::try_exists(&path).await.unwrap() {
                    if cog == "generic.private" {
                        let _ = msg
                    .reply(ctx, "Update failed because cannot move entry into generic.private when it contains attachments.")
                    .await;
                        return true;
                    }

                    Some(path)
                } else {
                    None
                }
            } else {
                None
            };

            if Some(&entry.name) == name.as_ref() {
                entries = vec![(entry, path)];
                break;
            }

            entries.push((entry, path));
        }

        let newname = if let Some(display) = newdisplay {
            let name = display.replace(" ", "-").to_lowercase();

            if name.parse::<i64>().is_ok() {
                let _ = msg
                    .reply(ctx, "Update failed because name cannot be an integer.")
                    .await;
                return true;
            }

            if name.chars().any(|c| !c.is_alphanumeric() && c != '-') {
                let _ = msg
                    .reply(
                        ctx,
                        "Update failed because name contains illegal characters.",
                    )
                    .await;
                return true;
            }

            if entries.len() > 1 {
                let _ = msg
                    .reply(ctx, "Update failed because cannot batch rename entries.")
                    .await;
                return true;
            }

            if let Some(found) = Coord::find_by_name(&name).await {
                if Some(found.id) != entries.first().map(|entry| entry.0.id) {
                    let _ = msg
                        .reply(
                            ctx,
                            "Update failed because a coord entry with that name already exists.",
                        )
                        .await;
                    return true;
                }
            }

            Some(name)
        } else {
            None
        };

        let newcog = if let Some(cog) = newcog {
            let (cog, cogid, subcogid) = if let Some(res) = Category::cogs_from_name(cog).await {
                res
            } else {
                let _ = msg
                    .reply(
                        ctx,
                        "Update failed because destination category does not exist.",
                    )
                    .await;
                return true;
            };

            let mut allowed = true;

            if let Some(cog) = cog {
                allowed &= Clearance::is_allowed(&cog.allowed, ctx, msg)
                    .await
                    .unwrap_or(true);

                if let Some(subcog) = cog.subcategories.get(&subcogid.unwrap_or(0).to_string()) {
                    allowed &= Clearance::is_allowed(&subcog.allowed, ctx, msg)
                        .await
                        .unwrap_or(true);
                }
            }

            if !allowed {
                let _ = msg
                    .reply(
                        ctx,
                        "Update failed because you don't have permission to write to the destination category.",
                    )
                    .await;
                return true;
            }

            Some((cogid, subcogid))
        } else {
            None
        };

        let newpos = if let Some(newpos) = newpos {
            let res = if let Some((x, z)) = newpos.split_once(',') {
                let x = x.parse::<i64>();
                let z = z.parse::<i64>();

                if x.is_err() || z.is_err() {
                    None
                } else {
                    Some((x.unwrap(), z.unwrap()))
                }
            } else {
                None
            };

            if res.is_none() {
                let _ = msg
                    .reply(
                        ctx,
                        "Update failed because could not parse new coordinates.",
                    )
                    .await;
                return true;
            }

            res
        } else {
            None
        };

        if newname.is_none()
            && newdesc.is_none()
            && newcog.is_none()
            && newdim.is_none()
            && newpos.is_none()
            && newtags.is_none()
        {
            let _ = msg
                .reply(ctx, "Update failed because no fields are changed.")
                .await;
            return true;
        }

        if entries.len() > 1 && newpos.is_some() {
            let _ = msg
                .reply(ctx, "Bulk editing location is not supported.")
                .await;
            return true;
        }

        if let Some((x, z)) = newpos {
            if let Some(entry) = Coord::find_near(
                x,
                z,
                unsafe { COORDS_CONFIG.get() }.unwrap().prevent_add_radius,
                newdim.unwrap(),
                ctx,
                msg,
            )
            .await
            {
                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "There is another entry nearby, consider updating **{}**{} instead.",
                            entry.display_name,
                            if entry.display_name == entry.name {
                                String::new()
                            } else {
                                format!(" ({})", entry.name)
                            }
                        ),
                    )
                    .await;
                return true;
            }
        }

        if newcog == Some((0, Some(1)))
            && !entries
                .iter()
                .all(|(entry, _)| entry.author_id == msg.author.id.get())
        {
            let _ = msg
                .reply(
                    ctx,
                    "Entries not moved to generic.private because you don't own all the entries.",
                )
                .await;
            return true;
        }

        let to_dir = if let Some(newcog) = newcog {
            Some(Category::path(newcog.0, newcog.1.unwrap_or_default()).await)
        } else {
            None
        };

        for (entry, path) in entries.iter_mut() {
            if let Some(name) = &newname {
                entry.name = name.clone();
                entry.display_name = newdisplay.unwrap().to_string();
            }

            if let Some(desc) = newdesc {
                entry.description = desc.to_string();
            }

            if let Some((x, z)) = newpos {
                entry.x = x;
                entry.z = z;
            }

            if let Some((cog, sub)) = newcog {
                entry.cog = cog;
                entry.subcog = sub.unwrap_or(0);
            }

            if let Some(dim) = newdim {
                entry.dim = dim;
            }

            if let Some(tags) = &newtags {
                entry.tags = tags.clone();
            }

            if let Some(path) = path {
                fs::rename(path, to_dir.as_ref().unwrap().join(entry.id.to_string()))
                    .await
                    .unwrap();
            }
        }

        for (entry, _) in entries.iter() {
            entry
                .save_replace(unsafe { COORDS.get() }.unwrap())
                .await
                .unwrap();
        }

        let _ = msg
            .reply(
                ctx,
                format!(
                    "{} {} updated.",
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
