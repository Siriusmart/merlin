use std::fmt::Write;

use mongodb::bson::doc;
use serenity::{
    all::{Context, Message},
    async_trait,
    futures::StreamExt,
};

use crate::{sys::Command, Clearance};

use super::{category::Category, collection::CATEGORIES};

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
                        format!("**Coords categories**{}", {
                            let s = unsafe { CATEGORIES.get() }
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
                                .await;

                            if s.is_empty() {
                                "There are no categories at the moment.".to_string()
                            } else {
                                s
                            }
                        }),
                    )
                    .await;
                return true;
            }
        };

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

                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "**[Coords category] {}.{}**{}\n{}",
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

        let _ = msg
            .reply(
                ctx,
                format!(
                    "**[Coords category] {}**{}\n{}\n\n**Subcategories**:{}",
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
                    if cog.subcategories.is_empty() {
                        "This category has no subcategories.".to_string()
                    } else {
                        let mut subcogs = cog.subcategories.values().collect::<Vec<_>>();
                        subcogs.sort_by_key(|val| &val.name);
                        subcogs.iter().fold(String::new(), |mut current, subcog| {
                            write!(current, "\n{} ({})", subcog.display_name, subcog.name).unwrap();
                            current
                        })
                    }
                ),
            )
            .await;

        true
    }
}
