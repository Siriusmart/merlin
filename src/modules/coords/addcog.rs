use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::sys::Command;

use super::category::Category;

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
            "[category]/[subcategory] (description)",
        ]
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool {
        match args {
            [name] if !name.contains('/') => addmain(name, "", ctx, msg).await,
            [name, description] if !name.contains('/') => {
                addmain(name, description, ctx, msg).await
            }
            _ => return false,
        }
        true
    }
}

async fn addmain(name: &str, desc: &str, ctx: &Context, msg: &Message) {
    let cog = Category::new(name.to_string(), desc.to_string()).await;

    match cog {
        Ok(_cog) => {
            let _ = msg.reply(ctx, "Category created!").await;
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
