use mongodb::bson::doc;
use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, CollectionItem, PerCommandConfig};

use super::{
    category::Category,
    collection::{CATEGORIES, COORDS},
};

pub struct CmdCogRm;

#[async_trait]
impl Command for CmdCogRm {
    fn name(&self) -> &str {
        "cogrm"
    }

    fn description(&self) -> &str {
        "Remove coords DB category."
    }

    fn usage(&self) -> &[&str] {
        &["[category]"]
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool {
        let (main, sub) = match args {
            [name, ..] if name.contains('.') => name
                .split_once('.')
                .map(|(left, right)| (left, Some(right)))
                .unwrap(),
            [name, ..] => (*name, None),
            _ => return false,
        };

        if main.to_lowercase().as_str() == "generic"
            || sub.is_some_and(|sub| sub.to_lowercase().as_str() == "unspecified")
        {
            let _ = msg.reply(ctx, "You cannot remove a system category.").await;
            return true;
        }

        let mut cog = if let Some(cog) = Category::get(main).await {
            cog
        } else {
            let _ = msg.reply(ctx, "Category not found.").await;
            return true;
        };

        let cog_display = cog.display_name.clone();
        let cog_name = cog.name.clone();

        if let Some(sub) = sub {
            let name = sub.replace(' ', "-").to_lowercase();

            let cog_allowed = cog.allowed.clone();

            let cogid = cog.id;

            if let Some(subcog) = cog.get_subcog_mut(&name) {
                if !Clearance::is_allowed(&subcog.allowed, ctx, msg)
                    .await
                    .unwrap_or(true)
                    || !Clearance::is_allowed(&cog_allowed, ctx, msg)
                        .await
                        .unwrap_or(true)
                {
                    let _ = msg
                        .reply(
                            ctx,
                            format!(
                                "You don't have permission to remove **{}.{}**{}.",
                                cog_display,
                                subcog.display_name,
                                if cog_name != cog_display || subcog.display_name != subcog.name {
                                    format!(" ({}.{})", cog_name, subcog.name)
                                } else {
                                    String::new()
                                }
                            ),
                        )
                        .await;
                    return true;
                }

                if unsafe { COORDS.get() }
                    .unwrap()
                    .find_one(doc! { "cog": cogid, "subcog": subcog.id})
                    .await
                    .unwrap()
                    .is_some()
                {
                    let _ = msg
                        .reply(ctx, "You cannot delete a nonempty category.")
                        .await;
                    return true;
                }

                cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
                    .await
                    .unwrap();

                let _ = msg.reply(ctx, "Category deleted.").await;

                return true;
            }
        }

        if !Clearance::is_allowed(&cog.allowed, ctx, msg)
            .await
            .unwrap_or(true)
        {
            let _ = msg
                .reply(
                    ctx,
                    format!(
                        "You don't have permission to remove **{}**{}.",
                        cog_display,
                        if cog_display == cog_name {
                            String::new()
                        } else {
                            format!(" ({cog_name})")
                        },
                    ),
                )
                .await;
            return true;
        }

        if unsafe { COORDS.get() }
            .unwrap()
            .find_one(doc! { "cog": cog.id})
            .await
            .unwrap()
            .is_some()
        {
            let _ = msg
                .reply(ctx, "You cannot delete a nonempty category.")
                .await;
            return true;
        }

        cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
            .await
            .unwrap();

        let _ = msg.reply(ctx, "Category deleted.").await;

        unsafe { CATEGORIES.get() }
            .unwrap()
            .delete_one(doc! { "_id": cog.id })
            .await
            .unwrap();

        true
    }

    fn percmd(&self) -> PerCommandConfig {
        PerCommandConfig {
            allowed: vec!["-everyone".to_string(), "?coorduser".to_string()],
            ..Default::default()
        }
    }
}
