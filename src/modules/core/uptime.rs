use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::sys::Command;

use super::keys::StartInstanceContainer;

pub struct CmdUptime;

#[async_trait]
impl Command for CmdUptime {
    fn name(&self) -> &str {
        "uptime"
    }

    fn description(&self) -> &str {
        "Check bot uptime."
    }

    fn usage(&self) -> &str {
        ""
    }

    async fn run(&self, _args: &[&str], ctx: &Context, msg: &Message) -> bool {
        let data = ctx.data.read().await;
        let elapsed = data
            .get::<StartInstanceContainer>()
            .unwrap()
            .get()
            .elapsed()
            .as_secs();

        let _ = msg
            .reply(
                ctx,
                format!("Bot has been up for {}", duration_string(elapsed)),
            )
            .await;

        true
    }
}

fn duration_string(mut sec: u64) -> String {
    if sec == 0 {
        return "less than a second".to_string();
    }

    const UNITS: &[(&str, u64)] = &[
        ("year", 31536000),
        ("month", 2419200),
        ("day", 86400),
        ("hour", 3600),
        ("minute", 60),
        ("second", 1),
    ];

    let mut out = vec![];

    for (unit, amount) in UNITS {
        let quantity = sec / amount;

        if quantity == 0 {
            continue;
        }

        sec -= quantity * amount;
        out.push(format!(
            "{quantity} {unit}{}",
            if quantity == 1 { "" } else { "s" }
        ));
    }

    if out.len() == 1 {
        out[0].clone()
    } else {
        format!(
            "{} and {}",
            out[0..out.len() - 1].join(", "),
            out.last().unwrap()
        )
    }
}
