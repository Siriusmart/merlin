use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, CommandHandler, MasterSwitch, PerCommandConfig};

pub struct CmdReload;

#[async_trait]
impl Command for CmdReload {
    fn name(&self) -> &str {
        "reload"
    }

    fn description(&self) -> &str {
        "Reload configurated options."
    }

    fn usage(&self) -> &[&str] {
        &[]
    }

    async fn run(&self, _args: &[&str], ctx: &Context, msg: &Message) -> bool {
        reload().await;

        let _ = msg.reply(ctx, "Config reloaded.").await;

        true
    }

    fn percmd(&self) -> PerCommandConfig {
        PerCommandConfig {
            allowed: vec!["-everyone".to_string()],
            ..Default::default()
        }
    }
}

async fn reload() {
    MasterSwitch::reload();
    Clearance::reload();
    #[cfg(feature = "mongo")]
    crate::Mongo::reload().await;
    CommandHandler::reload().await;
}
