use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, MasterOptions, MasterSwitch, PerCommandConfig};

pub struct CmdSave;

#[async_trait]
impl Command for CmdSave {
    fn name(&self) -> &str {
        "save"
    }

    fn description(&self) -> &str {
        "Write configurated options to file."
    }

    fn usage(&self) -> &[&str] {
        &[]
    }

    async fn run(&self, _args: &[&str], ctx: &Context, msg: &Message) -> bool {
        save();

        let _ = msg.reply(ctx, "Config saved.").await;

        true
    }

    fn percmd(&self) -> PerCommandConfig {
        PerCommandConfig {
            allowed: vec!["-everyone".to_string()],
            ..Default::default()
        }
    }
}

fn save() {
    MasterSwitch::write_to_config();
    MasterOptions::write_to_config();
    Clearance::write_to_config();
    // #[cfg(feature = "mongo")]
    // crate::Mongo::reload().await;
}
