use std::{collections::HashMap, sync::Arc};

use serenity::{
    all::{Context, Message},
    async_trait, Client,
};

use super::Command;

#[async_trait]
pub trait Module: Sync + Send + 'static {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn commands(&self) -> Arc<HashMap<String, Box<dyn Command>>>;

    fn default_command(&self) -> Option<Box<dyn Command>> {
        None
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) {
        if !args.is_empty() {
            if let Some(cmd) = self.commands().get(args[0]) {
                if !cmd.run(&args[1..], ctx, msg).await {
                    // TODO show correct syntax
                }

                return;
            }
        }

        if let Some(cmd) = self.default_command() {
            if !cmd.run(&args[1..], ctx, msg).await {
                // TODO show correct syntax
            }

            return;
        }

        // TODO show commands list
    }

    async fn setup(&mut self, _client: &Client) {}

    fn aliases(&self) -> &[(&str, &str)] {
        &[]
    }
}
