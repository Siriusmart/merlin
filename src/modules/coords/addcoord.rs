use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance};

use super::{
    category::Category,
    coord::{Coord, Dimension},
};

pub struct CmdAddCoord;

#[async_trait]
impl Command for CmdAddCoord {
    fn name(&self) -> &str {
        "addcoord"
    }

    fn description(&self) -> &str {
        "Add a coords item."
    }

    fn usage(&self) -> &[&str] {
        &["[name] (ow|nether|end) [x] [z] (category) (description)"]
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool {
        let (name, dim, x, z, cog) = match args {
            [name, dim, x, z, cog] if matches!(*dim, "ow" | "nether" | "end") => {
                (name, Some(dim), *x, *z, *cog)
            }
            [name, dim, x, z] if matches!(*dim, "ow" | "nether" | "end") => {
                (name, Some(dim), *x, *z, "generic.unspecified")
            }
            [name, x, z, cog] => (name, None, *x, *z, *cog),
            [name, x, z] => (name, None, *x, *z, "generic.unspecified"),
            _ => return false,
        };

        let dim = dim.map(|dim| Dimension::from_str(dim)).flatten();

        let (category, cog_id, subcog_id) =
            if let Some((category, cog, subcog)) = Category::cogs_from_name(cog).await {
                (category, cog, subcog)
            } else {
                let _ = msg
                    .reply(ctx, "Entry not added because the category does not exist.")
                    .await;
                return true;
            };

        if let Some(cog) = category {
            let subcog = cog.subcategories.get(&subcog_id.unwrap().to_string());
            if !Clearance::is_allowed(&cog.allowed, ctx, msg)
                .await
                .unwrap_or(true)
                || !(subcog.is_some()
                    || Clearance::is_allowed(&subcog.unwrap().allowed, ctx, msg)
                        .await
                        .unwrap_or(true))
            {
                let _ = msg
                    .reply(
                        ctx,
                        "You don't have permission to add entry to this category.",
                    )
                    .await;
                return true;
            }
        }

        let x = x.parse();
        let z = z.parse();

        if x.is_err() || z.is_err() {
            let _ = msg.reply(ctx, "Could not parse coordinates.").await;
            return true;
        }

        let entry = Coord::new(
            name.to_string(),
            args.get(5).map(|s| s.to_string()).unwrap_or_default(),
            msg.author.id.get(),
            cog_id,
            subcog_id.unwrap(),
            x.unwrap(),
            z.unwrap(),
            dim,
        )
        .await;

        match entry {
            Ok(_) => {
                let _ = msg.reply(ctx, "Entry added successfully.").await;
            }
            Err(e) => {
                let _ = msg
                    .reply(ctx, format!("Entry was not added because {e}."))
                    .await;
            }
        }

        true
    }
}
