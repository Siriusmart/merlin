use serenity::{
    all::{Context, Message},
    async_trait,
};

#[async_trait]
pub trait Command: Sync + Send {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool;
}
