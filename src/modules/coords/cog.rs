use std::fmt::Write;

use mongodb::bson::doc;
use serenity::{
    all::{Context, Message},
    async_trait,
    futures::StreamExt,
};

use crate::{sys::Command, Clearance, PerCommandConfig};

use super::{category::Category, collection::CATEGORIES, config::COORDS_CONFIG};

pub struct CmdCog;

#[async_trait]
impl Command for CmdCog {
    fn name(&self) -> &str {
        "cog"
    }

    fn description(&self) -> &str {
        "Show coords DB category."
    }

    fn usage(&self) -> &[&str] {
        &["[category]", "[category].[subcategory]"]
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool {
        let (main, sub) = match args {
            [name] if name.contains('.') => name
                .split_once('.')
                .map(|(left, right)| (left, Some(right)))
                .unwrap(),
            [name] => (*name, None),
            _ => {
                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "**Coords categories**\n\\- generic{}\n\nAttachment path: `{}`",
                            {
                                unsafe { CATEGORIES.get() }
                                    .unwrap()
                                    .find(doc! {})
                                    .await
                                    .unwrap()
                                    .map(|item| item.unwrap())
                                    .map(|item| {
                                        format!(
                                            "\n\\- {}{}",
                                            item.display_name,
                                            if item.display_name != item.name {
                                                format!(" ({})", item.name)
                                            } else {
                                                String::new()
                                            }
                                        )
                                    })
                                    .collect::<String>()
                                    .await
                            },
                            unsafe { COORDS_CONFIG.get() }
                                .unwrap()
                                .default_attachment_path
                        ),
                    )
                    .await;
                return true;
            }
        };

        let sublower = sub.map(str::to_lowercase).unwrap_or(String::new());

        match (main.to_lowercase().as_str(), sublower.as_str()) {
            ("generic", "") => {
                let _ = msg.reply(ctx, format!("**[Coords category] generic**\nSystem categories of special function.\n\n**Subcategories**\n\\- unspecified\n\\- private\n\nAttachment path: `{}` (inherited)", unsafe { COORDS_CONFIG.get() }.unwrap().default_attachment_path)).await;
                return true;
            }
            ("generic", "private") => {
                let _ = msg.reply(ctx, "**[Coords category] generic.private**\nOnly the author can see entries in this category.").await;
                return true;
            }
            (main, "unspecified") => {
                let cog = if let Some(cog) = Category::get(main).await {
                    cog
                } else {
                    let _ = msg.reply(ctx, "Category not found.").await;
                    return true;
                };
                let path = cog.attachment_path.as_ref().unwrap_or(
                    &unsafe { COORDS_CONFIG.get() }
                        .unwrap()
                        .default_attachment_path,
                );

                let _ = msg.reply(ctx, format!("**[Coords category] {main}.unspecified**\nThe default subcategory for {main}.\n\nAttachment path: `{path}` (inherited)")).await;
                return true;
            }
            _ => {}
        }

        let cog = if let Some(cog) = Category::get(main).await {
            cog
        } else {
            let _ = msg.reply(ctx, "Category not found.").await;
            return true;
        };

        if let Some(sub) = sub {
            let name = sub.replace(' ', "-").to_lowercase();

            if let Some(subcog) = cog.get_subcog(&name) {
                if !Clearance::is_allowed(&subcog.allowed, ctx, msg)
                    .await
                    .unwrap_or(true)
                    || !Clearance::is_allowed(&cog.allowed, ctx, msg)
                        .await
                        .unwrap_or(true)
                {
                    let _ = msg
                        .reply(
                            ctx,
                            format!(
                                "You don't have permission to view **{}.{}**{}.",
                                cog.display_name,
                                subcog.display_name,
                                if cog.name != cog.display_name
                                    || subcog.display_name != subcog.name
                                {
                                    format!(" ({}.{})", cog.name, subcog.name)
                                } else {
                                    String::new()
                                }
                            ),
                        )
                        .await;
                    return true;
                }

                let path = subcog.attachment_path.as_ref().unwrap_or(
                    &unsafe { COORDS_CONFIG.get() }
                        .unwrap()
                        .default_attachment_path,
                );

                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "**[Coords category] {}.{}**{}\n{}\n\nAttachment path: `{path}`{}",
                            cog.display_name,
                            subcog.display_name,
                            if cog.name != cog.display_name || subcog.display_name != subcog.name {
                                format!(" ({}.{})", cog.name, subcog.name)
                            } else {
                                String::new()
                            },
                            if subcog.description.is_empty() {
                                "This category has no description."
                            } else {
                                subcog.description.as_str()
                            },
                            if subcog.attachment_path.is_none() {
                                " (inherited)"
                            } else {
                                ""
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
                        "You don't have permission to view **{}**{}.",
                        cog.display_name,
                        if cog.display_name == cog.name {
                            String::new()
                        } else {
                            format!(" ({})", cog.name)
                        },
                    ),
                )
                .await;
            return true;
        }

        let path = cog.attachment_path.as_ref().unwrap_or(
            &unsafe { COORDS_CONFIG.get() }
                .unwrap()
                .default_attachment_path,
        );

        let _ = msg
            .reply(
                ctx,
                format!(
                    "**[Coords category] {}**{}\n{}\n\n**Subcategories**:\n\\- unspecified{}\n\nPath: `{path}`{}",
                    cog.display_name,
                    if cog.name == cog.display_name {
                        String::new()
                    } else {
                        format!(" ({})", cog.name)
                    },
                    if cog.description.is_empty() {
                        "This category has no description."
                    } else {
                        cog.description.as_str()
                    },
                    {
                        let mut subcogs = cog.subcategories.values().collect::<Vec<_>>();
                        subcogs.sort_by_key(|val| &val.name);
                        subcogs.iter().fold(String::new(), |mut current, subcog| {
                            write!(
                                current,
                                "\n\\- {}{}",
                                subcog.display_name,
                                if subcog.name == subcog.display_name {
                                    String::new()
                                } else {
                                    format!(" ({})", subcog.name)
                                }
                            )
                            .unwrap();
                            current
                        })
                    },
                    if cog.attachment_path.is_none() {
                        " (inherited)"
                    } else {
                        ""
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
