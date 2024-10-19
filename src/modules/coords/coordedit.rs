use std::collections::HashMap;

use mongodb::bson::Document;
use serenity::{
    all::{Context, Message},
    async_trait,
    futures::StreamExt,
};

use crate::{sys::Command, Clearance, CollectionItem};

use super::{
    category::Category,
    collection::COORDS,
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
            "(name|*) (cog=value|near=x,z,radius|dim=ow/nether/end...) (newname=value|newdesc=value|newcog=value|newpos=x,z|newdim=ow/nether/end...)",
        ]
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

        let mut near: Option<(i64, i64, i64)> = None;

        let mut newdisplay = None;
        let mut newdesc = None;
        let mut newcog = None;
        let mut newpos = None;
        let mut newdim = None;

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
                    "newname" => newdisplay = Some(right),
                    "newdesc" => newdesc = Some(right),
                    "newcog" => newcog = Some(right),
                    "newpos" => newpos = Some(right),
                    "newdim" if matches!(right, "ow" | "nether" | "end") => {
                        newdim = Some(Dimension::from_str(right).unwrap())
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
                if (x - entry.x).pow(2) + z.pow(2) > r2 {
                    continue;
                }
            }

            entries.push(entry);
        }

        let newname = if let Some(display) = newdisplay {
            let name = display.replace(" ", "-");

            if name.chars().any(|c| !c.is_alphanumeric() && c != '-') {
                let _ = msg
                    .reply(
                        ctx,
                        "Update failed because name contains illegal characters.",
                    )
                    .await;
                return true;
            }

            if entries.len() > 1 || Coord::find_by_name(&name).await.is_some() {
                let _ = msg
                    .reply(
                        ctx,
                        "Update failed because a coord entry with that name already exists.",
                    )
                    .await;
                return true;
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
            let res = if let Some((x, z)) = newpos.split_once('.') {
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

        if newname.is_none() && newdesc.is_none() && newcog.is_none() && newdim.is_none() {
            let _ = msg
                .reply(ctx, "Update failed because no fields are changed.")
                .await;
            return true;
        }

        for entry in entries.iter_mut() {
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

            entry
                .save_replace(unsafe { COORDS.get() }.unwrap())
                .await
                .unwrap();
        }

        let _ = msg
            .reply(
                ctx,
                format!(
                    "{} entrie{} updated.",
                    entries.len(),
                    if entries.len() > 1 { "s" } else { "" }
                ),
            )
            .await;

        true
    }
}
