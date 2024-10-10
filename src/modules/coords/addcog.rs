use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, CollectionItem};

use super::{
    category::{Category, Subcategory},
    collection::CATEGORIES,
};

pub struct CmdAddcog;

#[async_trait]
impl Command for CmdAddcog {
    fn name(&self) -> &str {
        "addcog"
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
    let cog = Category::new(name.to_string(), desc.to_string()).await;

    match cog {
        Ok(cog) if cog.name == name => {
            let _ = msg
                .reply(ctx, format!("Category **{}** created!", cog.name))
                .await;
        }
        Ok(cog) => {
            let _ = msg
                .reply(
                    ctx,
                    format!("Category **{}** ({}) created!", cog.display_name, cog.name),
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

    if cog.subcategories.contains_key(sub) {
        let _ = msg.reply(ctx, "Subcategory already exist.").await;
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

    let subcog = Subcategory::new(name.to_string(), desc.to_string());
    let real_name = subcog.name.clone();

    cog.subcategories.insert(name.clone(), subcog);
    cog.save_replace(unsafe { CATEGORIES.get() }.unwrap())
        .await
        .unwrap();

    let _ = msg
        .reply(
            ctx,
            format!("Subcategory **{}.{}** created!", cog.name, real_name),
        )
        .await;
}
