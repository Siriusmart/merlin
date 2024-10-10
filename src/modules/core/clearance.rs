use std::fmt::Write;

use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, PerCommandConfig};

pub struct CmdClearance;

#[async_trait]
impl Command for CmdClearance {
    fn name(&self) -> &str {
        "clearance"
    }

    fn description(&self) -> &str {
        "Manage clearance presets."
    }

    fn usage(&self) -> &[&str] {
        &["(preset)", "[preset] (rules...)", "[preset] clear"]
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool {
        match args {
            [] => {
                let mut presets = Clearance::list_all();
                presets.sort();

                if presets.is_empty() {
                    let _ = msg
                        .reply(
                            ctx,
                            "**Clearance presets**\nThere are no clearance presets.",
                        )
                        .await;
                    return true;
                }

                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "**Clearance presets**{}",
                            presets.iter().fold(String::new(), |mut current, label| {
                                write!(current, "\n\\- {}", label).unwrap();
                                current
                            })
                        ),
                    )
                    .await;
            }
            [preset] => {
                let rules = Clearance::get(preset);

                if rules.is_empty() {
                    let _ = msg
                        .reply(
                            ctx,
                            format!("**[Clearance preset] {preset}**\nClearance preset *{preset}* has no rules.",),
                        )
                        .await;
                } else {
                    let _ = msg
                        .reply(
                            ctx,
                            format!(
                                "**[Clearance preset] {preset}**{}",
                                rules.iter().enumerate().fold(
                                    String::new(),
                                    |mut current, (index, rule)| {
                                        write!(current, "\n{}\\. {}", index + 1, rule).unwrap();
                                        current
                                    }
                                )
                            ),
                        )
                        .await;
                }
            }
            [preset, "clear"] => {
                if Clearance::remove(preset) {
                    let _ = msg
                        .reply(
                            ctx,
                            format!(
                                "Clearance preset **{preset}** has been clearned. *(not saved)*",
                            ),
                        )
                        .await;
                } else {
                    let _ = msg
                        .reply(
                            ctx,
                            format!("Clearance preset **{preset}** has been clearned, but is was originally empty.",),
                        )
                        .await;
                }
            }
            [preset, ..] => {
                if args.iter().skip(1).any(|line| line.starts_with('?'))
                    || !Clearance::validate(&args[1..])
                {
                    let _ = msg
                        .reply(
                            ctx,
                            format!("Failed to update clearance preset **{preset}** because it contains invalid rules.",),
                        )
                        .await;
                } else {
                    Clearance::set(preset.to_string(), &args[1..]);
                    let _ = msg
                        .reply(ctx, format!("Clearance preset **{preset}** updated.",))
                        .await;
                }
            }
        }

        true
    }

    fn percmd(&self) -> PerCommandConfig {
        PerCommandConfig {
            allowed: vec!["?admin".to_string()],
            ..Default::default()
        }
    }
}
