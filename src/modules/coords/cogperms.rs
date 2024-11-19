use std::fmt::Write;

use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, CollectionItem, PerCommandConfig};

use super::{category::Category, collection::CATEGORIES};

pub struct CmdCogPerms;

#[async_trait]
impl Command for CmdCogPerms {
    fn name(&self) -> &str {
        "cogperms"
    }

    fn description(&self) -> &str {
        "Edit coords DB category permissions."
    }

    fn usage(&self) -> &[&str] {
        &["[category] (rules...)", "[category] clear"]
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

                if args.len() == 1 {
                    display_perms(
                        &subcog.allowed,
                        &subcog.name,
                        &subcog.display_name,
                        ctx,
                        msg,
                    )
                    .await;
                    return true;
                } else if args[1..].as_ref() == ["clear"] {
                    subcog.allowed.clear();

                    cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
                        .await
                        .unwrap();

                    let _ = msg.reply(ctx, "Category permissions cleared.").await;
                } else {
                    if !Clearance::validate(&args[1..], None) {
                        let _ = msg
                            .reply(
                                ctx,
                                "Failed to update category permission because it contains invalid rules.".to_string(),
                            )
                            .await;
                        return true;
                    }

                    subcog.allowed = args[1..].iter().map(|s| s.to_string()).collect();
                    Clearance::map_rules(&mut subcog.allowed, msg, ctx).await;

                    cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
                        .await
                        .unwrap();

                    let _ = msg.reply(ctx, "Category permissions updated.").await;
                }

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

        if args.len() == 1 {
            display_perms(&cog.allowed, &cog.name, &cog.display_name, ctx, msg).await;
            return true;
        } else if args[1..].as_ref() == ["clear"] {
            cog.allowed.clear();

            cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
                .await
                .unwrap();

            let _ = msg.reply(ctx, "Category permissions cleared.").await;
        } else {
            if !Clearance::validate(&args[1..], None) {
                let _ = msg
                    .reply(
                        ctx,
                        "Failed to update category permission because it contains invalid rules."
                            .to_string(),
                    )
                    .await;
                return true;
            }

            cog.allowed = args[1..].iter().map(|s| s.to_string()).collect();
            Clearance::map_rules(&mut cog.allowed, msg, ctx).await;

            cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
                .await
                .unwrap();

            let _ = msg.reply(ctx, "Category details updated.").await;
        }

        true
    }

    fn percmd(&self) -> PerCommandConfig {
        PerCommandConfig {
            allowed: vec!["-everyone".to_string(), "?coordmod".to_string()],
            ..Default::default()
        }
    }
}

async fn display_perms(
    allowed: &[String],
    name: &str,
    display: &str,
    ctx: &Context,
    msg: &Message,
) {
    if allowed.is_empty() {
        let _ = msg
            .reply(
                ctx,
                format!(
                    "**[Category permission] {}{}**\nThis module has no permission rules.",
                    display,
                    if display == name { "" } else { name }
                ),
            )
            .await;
        return;
    }

    let _ = msg
        .reply(
            ctx,
            format!(
                "**[Category permission] {}{}**{}",
                display,
                if display == name { "" } else { name },
                allowed
                    .iter()
                    .enumerate()
                    .fold(String::new(), |mut current, (index, rule)| {
                        write!(current, "\n{}\\. {}", index + 1, rule).unwrap();
                        current
                    })
            ),
        )
        .await;
}
