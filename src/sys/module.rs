use std::{collections::HashMap, sync::Arc};

use serenity::{
    all::{Context, Message},
    async_trait,
};

use super::{Command, CommandHandler, MasterSwitch, PerCommandConfig, PerModuleConfig};

#[async_trait]
pub trait Module: Sync + Send + 'static {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn commands(&self) -> Arc<HashMap<String, Box<dyn Command>>>;

    fn default_command(&self) -> Option<String> {
        Some(format!("help {}", self.name()))
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) {
        if !args.is_empty() {
            if let Some(cmd) = self.commands().get(args[0].to_lowercase().as_str()) {
                let permod = MasterSwitch::get(self.name()).unwrap();

                if !permod.is_allowed(ctx, msg).await
                    || !permod
                        .commands
                        .get(args[0])
                        .unwrap()
                        .is_allowed(ctx, msg)
                        .await
                {
                    return;
                }

                if !cmd.run(&args[1..], ctx, msg).await {
                    CommandHandler::help(&[&[self.name()], args].concat(), ctx, msg).await
                }

                return;
            }
        }

        if let Some(cmd) = self.default_command() {
            CommandHandler::run(
                shell_words::split(&cmd)
                    .unwrap()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .as_ref(),
                ctx,
                msg,
            )
            .await;
            return;
        }

        CommandHandler::help(&[self.name()], ctx, msg).await
    }

    async fn setup(&mut self) {}
    async fn reload(&mut self) {}

    fn aliases(&self) -> &[(&str, &str)] {
        &[]
    }

    fn permod(&self) -> PerModuleConfig {
        PerModuleConfig {
            commands: self.percmds(),
            ..Default::default()
        }
    }

    fn percmds(&self) -> HashMap<String, PerCommandConfig> {
        let mut out = HashMap::new();

        for (cmd_label, cmd) in self.commands().iter() {
            out.insert(cmd_label.to_string(), cmd.percmd());
        }

        out
    }
}
