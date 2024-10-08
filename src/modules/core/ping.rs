use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::sys::Command;

use super::keys::ShardManagerContainer;

pub struct CmdPing;

#[async_trait]
impl Command for CmdPing {
    fn name(&self) -> &str {
        "ping"
    }

    fn description(&self) -> &str {
        "Check shard network latency."
    }

    fn usage(&self) -> &[&str] {
        &[]
    }

    async fn run(&self, _args: &[&str], ctx: &Context, msg: &Message) -> bool {
        let data = ctx.data.read().await;

        let shard_manager = match data.get::<ShardManagerContainer>() {
            Some(v) => v,
            None => {
                let _ = msg
                    .reply(ctx, "There was a problem getting the shard manager")
                    .await;

                return true;
            }
        };

        let runners = shard_manager.runners.lock().await;

        let runner = match runners.get(&ctx.shard_id) {
            Some(runner) => runner,
            None => {
                let _ = msg.reply(ctx, "No shard found").await;

                return true;
            }
        };

        let _ = msg
            .reply(
                ctx,
                format!(
                    "The shard latency is {}",
                    runner
                        .latency
                        .map(|dur| format!("{}ms", dur.as_millis()))
                        .unwrap_or("not yet known".to_string())
                ),
            )
            .await;

        true
    }
}
