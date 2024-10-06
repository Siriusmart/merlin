use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::sys::Command;

pub struct CmdVersion;

#[async_trait]
impl Command for CmdVersion {
    fn name(&self) -> &str {
        "version"
    }

    fn description(&self) -> &str {
        "Check bot version."
    }

    fn usage(&self) -> &str {
        ""
    }

    async fn run(&self, _args: &[&str], ctx: &Context, msg: &Message) -> bool {
        let _ = msg
            .reply(
                ctx,
                format!(
                    "Running {} {} (Git {})",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                    env!("GIT_HASH")
                ),
            )
            .await;

        true
    }
}
