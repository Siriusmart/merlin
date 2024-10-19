use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, PerCommandConfig};

use super::{
    category::Category,
    config::COORDS_CONFIG,
    coord::{Coord, Dimension},
};

pub struct CmdCoordAdd;

#[async_trait]
impl Command for CmdCoordAdd {
    fn name(&self) -> &str {
        "coordadd"
    }

    fn description(&self) -> &str {
        "Add a coords item."
    }

    fn usage(&self) -> &[&str] {
        &["[name] [ow|nether|end] [x] [z] (category) (description)"]
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool {
        let (name, dim, x, z, cog, desc) = match args {
            [name, dim, x, z, cog, desc] if matches!(*dim, "ow" | "nether" | "end") => {
                (name, dim, *x, *z, *cog, *desc)
            }
            [name, dim, x, z, cog] if matches!(*dim, "ow" | "nether" | "end") => {
                (name, dim, *x, *z, *cog, "")
            }
            [name, dim, x, z] if matches!(*dim, "ow" | "nether" | "end") => {
                (name, dim, *x, *z, "generic.unspecified", "")
            }
            _ => return false,
        };

        let dim = if let Some(dim) = Dimension::from_str(dim) {
            dim
        } else {
            return false;
        };

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
            let subcog = cog.subcategories.get(&subcog_id.unwrap_or(0).to_string());
            if !Clearance::is_allowed(&cog.allowed, ctx, msg)
                .await
                .unwrap_or(true)
                || !(subcog.is_none()
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

        let x = x.unwrap();
        let z = z.unwrap();

        if let Some(entry) = Coord::find_near(
            x,
            z,
            unsafe { COORDS_CONFIG.get() }.unwrap().prevent_add_radius,
            dim,
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

        let entry = Coord::new(
            name.to_string(),
            desc.to_string(),
            msg.author.id.get(),
            cog_id,
            subcog_id.unwrap_or(0),
            x,
            z,
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

    fn percmd(&self) -> PerCommandConfig {
        PerCommandConfig {
            allowed: vec!["-everyone".to_string(), "?coorduser".to_string()],
            ..Default::default()
        }
    }
}
