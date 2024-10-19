use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, CollectionItem};

use super::{
    category::{Category, Subcategory},
    collection::CATEGORIES,
};

pub struct CmdCogAdd;

#[async_trait]
impl Command for CmdCogAdd {
    fn name(&self) -> &str {
        "cogadd"
    }

    fn description(&self) -> &str {
        "Add coords DB category."
    }

    fn usage(&self) -> &[&str] {
        &[
            "[category] (description)",
            "[category].[subcategory] (description)",
        ]
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool {
        match args {
            [name] if !name.contains('.') => addmain(name, "", ctx, msg).await,
            [name, description] if !name.contains('.') => {
                addmain(name, description, ctx, msg).await
            }
            [name] => addsub(name, "", ctx, msg).await,
            [name, description] => addsub(name, description, ctx, msg).await,
            _ => return false,
        }
        true
    }
}

async fn addmain(name: &str, desc: &str, ctx: &Context, msg: &Message) {
    if name.is_empty() {
        let _ = msg.reply(ctx, "Category name cannot be empty.").await;
        return;
    }

    if name == "generic" || Category::get(name).await.is_some() {
        let _ = msg
            .reply(
                ctx,
                "Category not created because a category with that name already exist.",
            )
            .await;
        return;
    }

    let cog = Category::new(name.to_string(), desc.to_string()).await;

    match cog {
        Ok(cog) => {
            let _ = msg
                .reply(
                    ctx,
                    format!(
                        "Category **{}**{} created!",
                        cog.display_name,
                        if cog.name == cog.display_name {
                            String::new()
                        } else {
                            format!(" ({})", cog.name)
                        }
                    ),
                )
                .await;
        }
        Err(reason) => {
            let _ = msg
                .reply(
                    ctx,
                    format!("Could not create new category because {reason}."),
                )
                .await;
        }
    }
}

async fn addsub(name: &str, desc: &str, ctx: &Context, msg: &Message) {
    let (main, sub) = name.split_once('.').unwrap();

    if sub.contains('.') {
        let _ = msg
            .reply(ctx, "The maximum depth for nested categories is 2.")
            .await;
        return;
    }

    let mut cog = if let Some(cog) = Category::get(main).await {
        cog
    } else {
        let _ = msg.reply(ctx, "Parent category not found.").await;
        return;
    };

    if !Clearance::is_allowed(&cog.allowed, ctx, msg)
        .await
        .unwrap_or(true)
    {
        let _ = msg
            .reply(
                ctx,
                format!(
                    "You don't have permission to edit **{}**{}.",
                    cog.display_name,
                    if cog.name == cog.display_name {
                        String::new()
                    } else {
                        format!(" ({})", cog.name)
                    }
                ),
            )
            .await;
        return;
    }

    let name = sub.replace(' ', "-").to_lowercase();

    if name.chars().any(|c| !c.is_alphanumeric() && c != '-') {
        let _ = msg
            .reply(
                ctx,
                "Could not create new subcategory because name contains illegal characters.",
            )
            .await;
        return;
    }

    if name.is_empty() {
        let _ = msg.reply(ctx, "Category name cannot be empty.").await;
        return;
    }

    if name == "unspecified" || cog.contains(&name) {
        let _ = msg
            .reply(
                ctx,
                "Category not created because a category with that name already exist.",
            )
            .await;
        return;
    }

    let subcog = Subcategory::new(name.to_string(), sub.to_string(), desc.to_string(), cog.id);

    cog.subcategories
        .insert(cog.subcogcounter.to_string(), subcog);
    cog.subcogcounter += 1;
    cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
        .await
        .unwrap();

    let _ = msg
        .reply(
            ctx,
            format!(
                "Subcategory **{}.{}**{} created!",
                cog.display_name,
                sub,
                if cog.name != cog.display_name || sub != name {
                    format!(" ({}.{})", cog.name, sub)
                } else {
                    String::new()
                }
            ),
        )
        .await;
}
