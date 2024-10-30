use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, CollectionItem, PerCommandConfig};

use super::{category::Category, collection::CATEGORIES, config::COORDS_CONFIG};

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
        &["[category] [desc=value|name=value|path=value...]"]
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
        let mut new_path = None;

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
                    "path" => new_path = Some(right),
                    _ => return false,
                }
            } else {
                return false;
            }
        }

        if new_path.is_none() && new_name.is_none() && new_desc.is_none() {
            let _ = msg
                .reply(ctx, "Update failed because no fields are changed.")
                .await;
            return true;
        }

        let mut cog = if let Some(cog) = Category::get(main).await {
            cog
        } else {
            let _ = msg.reply(ctx, "Category not found.").await;
            return true;
        };

        let cog2 = cog.clone();

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
                                cog2.display_name,
                                subcog.display_name,
                                if cog2.name != cog2.display_name
                                    || subcog.display_name != subcog.name
                                {
                                    format!(" ({}.{})", cog2.name, subcog.name)
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

                if let Some(path) = new_path {
                    let to = if path.is_empty() {
                        None
                    } else {
                        Some(path.to_string())
                    };

                    Category::move_all(
                        &cog2,
                        Some(subcog.id),
                        subcog.attachment_path.as_ref().unwrap_or(
                            cog2.attachment_path.as_ref().unwrap_or(
                                &unsafe { COORDS_CONFIG.get() }
                                    .unwrap()
                                    .default_attachment_path,
                            ),
                        ),
                        to.as_ref().unwrap_or(
                            cog2.attachment_path.as_ref().unwrap_or(
                                &unsafe { COORDS_CONFIG.get() }
                                    .unwrap()
                                    .default_attachment_path,
                            ),
                        ),
                    )
                    .await;

                    subcog.attachment_path = to;
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
                        cog2.display_name,
                        if cog2.display_name == cog2.name {
                            String::new()
                        } else {
                            format!(" ({})", cog2.name)
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

        if let Some(path) = new_path {
            let to = if path.is_empty() {
                None
            } else {
                Some(path.to_string())
            };

            Category::move_all(
                &cog2,
                None,
                cog.attachment_path.as_ref().unwrap_or(
                    &unsafe { COORDS_CONFIG.get() }
                        .unwrap()
                        .default_attachment_path,
                ),
                to.as_ref().unwrap_or(
                    &unsafe { COORDS_CONFIG.get() }
                        .unwrap()
                        .default_attachment_path,
                ),
            )
            .await;

            cog.attachment_path = to;
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
