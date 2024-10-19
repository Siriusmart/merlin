use mongodb::bson::doc;
use serenity::{
    all::{Context, Message},
    async_trait,
    futures::StreamExt,
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

                let mut entries = unsafe { COORDS.get() }
                    .unwrap()
                    .find(doc! { "cog": cogid, "subcog": subcog.id})
                    .await
                    .unwrap()
                    .map(Result::unwrap)
                    .collect::<Vec<_>>()
                    .await;

                for entry in entries.iter_mut() {
                    entry.subcog = 0;
                    entry
                        .save_replace(unsafe { COORDS.get() }.unwrap())
                        .await
                        .unwrap();
                }

                cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
                    .await
                    .unwrap();

                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "Category deleted{}.",
                            if entries.is_empty() {
                                String::new()
                            } else {
                                format!(
                                    ", {} {} been moved to {}.unspecified{}",
                                    entries.len(),
                                    if entries.len() == 1 {
                                        "entry has"
                                    } else {
                                        "entries have"
                                    },
                                    cog.display_name,
                                    if cog.name != cog.display_name {
                                        format!(" ({}.unspecified)", cog.name)
                                    } else {
                                        String::new()
                                    }
                                )
                            }
                        ),
                    )
                    .await;

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

        let mut entries = unsafe { COORDS.get() }
            .unwrap()
            .find(doc! { "cog": cog.id})
            .await
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .await;

        for entry in entries.iter_mut() {
            entry.subcog = 0;
            entry
                .save_replace(unsafe { COORDS.get() }.unwrap())
                .await
                .unwrap();
        }

        cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
            .await
            .unwrap();

        let _ = msg
            .reply(
                ctx,
                format!(
                    "Category deleted{}.",
                    if entries.is_empty() {
                        String::new()
                    } else {
                        format!(
                            ", {} {} been moved to generic.unspecified",
                            entries.len(),
                            if entries.len() == 1 {
                                "entry has"
                            } else {
                                "entries have"
                            }
                        )
                    }
                ),
            )
            .await;

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
