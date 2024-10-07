use merlin::{CommandHandler, Config, MasterOptions, MasterSwitch, MASTER};
use serenity::{all::*, async_trait, Client};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let master = unsafe { MASTER.get() }.unwrap();
        if msg.content.starts_with(master.prefix.as_str()) {
            if let Ok(args) = shell_words::split(&msg.content[master.prefix.len()..]) {
                CommandHandler::run(
                    args.iter().map(String::as_str).collect::<Vec<_>>().as_ref(),
                    &ctx,
                    &msg,
                )
                .await;
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!(
            "{}#{} is connected! (shard #{})",
            ready.user.name,
            ready
                .user
                .discriminator
                .map(|n| n.get())
                .unwrap_or_default(),
            ctx.shard_id
        );
    }
}

#[tokio::main]
async fn main() {
    MasterOptions::setup();
    let mut switch = MasterSwitch::load();
    let intents = GatewayIntents::all();

    #[cfg(feature = "mongo")]
    merlin::Mongo::load().await;

    let client = Client::builder(unsafe { MASTER.get() }.unwrap().token.as_str(), intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    CommandHandler::client_set(client);

    CommandHandler::load(&mut switch).await;
    switch.finalise();

    if let Err(e) = CommandHandler::client_mut().start().await {
        println!("Client error: {e:?}");
    }
}
