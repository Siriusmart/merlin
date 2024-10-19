use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, CollectionItem, PerCommandConfig};

use super::{category::Category, collection::CATEGORIES};

pub struct CmdCogEdit;

#[async_trait]
impl Command for CmdCogEdit {
    fn name(&self) -> &str {
        "cogedit"
    }

    fn description(&self) -> &str {
        "Edit coords DB category details."
    }

    fn usage(&self) -> &[&str] {
        &["[category] [desc=value|name=value...]"]
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
            let _ = msg.reply(ctx, "You cannot edit a system category.").await;
            return true;
        }

        let mut new_desc = None;
        let mut new_name = None;
        let mut new_display = None;

        for arg in args[1..].iter() {
            if let Some((left, right)) = arg.split_once('=') {
                match left {
                    "desc" => new_desc = Some(right),
                    "name" => {
                        if right.is_empty() {
                            let _ = msg.reply(ctx, "Category name cannot be empty.").await;
                            return true;
                        }

                        let name = right.replace(' ', "-").to_lowercase();

                        if name.chars().any(|c| !c.is_alphanumeric() && c != '-') {
                            let _ = msg.reply(ctx, "Category details not updated because name contains illegal characters.").await;
                            return true;
                        }

                        new_name = Some(name);
                        new_display = Some(right);
                    }
                    _ => return false,
                }
            } else {
                return false;
            }
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
            let is_duplicate = new_name.as_ref().is_some_and(|name| cog.contains(name));

            let cog_allowed = cog.allowed.clone();

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
                                "You don't have permission to edit **{}.{}**{}.",
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

                if is_duplicate {
                    let _ = msg
                        .reply(
                            ctx,
                            "Category not updated because a category with that name already exist.",
                        )
                        .await;
                    return true;
                }

                if let Some(name) = new_name {
                    subcog.name = name;
                    subcog.display_name = new_display.unwrap().to_string();
                }

                if let Some(desc) = new_desc {
                    subcog.description = desc.to_string();
                }

                cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
                    .await
                    .unwrap();

                let _ = msg.reply(ctx, "Category details updated.").await;

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
                        "You don't have permission to edit **{}**{}.",
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

        let is_duplicate = if let Some(name) = &new_name {
            Category::get(name).await.is_some()
        } else {
            false
        };

        if is_duplicate {
            let _ = msg
                .reply(
                    ctx,
                    "Category not updated because a category with that name already exist.",
                )
                .await;
            return true;
        }

        if let Some(name) = new_name {
            cog.name = name;
            cog.display_name = new_display.unwrap().to_string();
        }

        if let Some(desc) = new_desc {
            cog.description = desc.to_string();
        }

        cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
            .await
            .unwrap();

        let _ = msg.reply(ctx, "Category details updated.").await;

        true
    }

    fn percmd(&self) -> PerCommandConfig {
        PerCommandConfig {
            allowed: vec!["-everyone".to_string(), "?coorduser".to_string()],
            ..Default::default()
        }
    }
}
