use std::fmt::Write;

use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, MasterSwitch, PerCommandConfig};

pub struct CmdSwitch;

#[async_trait]
impl Command for CmdSwitch {
    fn name(&self) -> &str {
        "switch"
    }

    fn description(&self) -> &str {
        "Enable/disable commands and modules."
    }

    fn usage(&self) -> &[&str] {
        &["[module] (enable|disable)"]
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool {
        match args {
            ["core.switch", val] if matches!(*val, "enable" | "disable") => {
                let _ = msg.reply(ctx, "core.switch cannot be disabled.").await;
            }
            ["core", val] if matches!(*val, "enable" | "disable") => {
                let _ = msg.reply(ctx, "core cannot be disabled.").await;
            }
            [item, val] => {
                if !matches!(*val, "enable" | "disable") {
                    return false;
                }

                let value = *val == "enable";

                let success = match item.split_once('.') {
                    Some((module, cmd)) => MasterSwitch::switch(module, Some(cmd), value),
                    None => MasterSwitch::switch(item, None, value),
                };

                if success {
                    let _ = msg
                        .reply(ctx, format!("{item} has been {val}d. *(not saved)*"))
                        .await;
                } else {
                    let _ = msg.reply(ctx, "No such module.").await;
                }
            }
            [item] => match item.split_once('.') {
                Some((module_str, cmd_str)) => {
                    let module = match MasterSwitch::get(module_str) {
                        Some(module) => module,
                        None => {
                            let _ = msg.reply(ctx, "No such module.").await;
                            return true;
                        }
                    };

                    let cmd = match module.commands.get(cmd_str) {
                        Some(cmd) => cmd,
                        None => {
                            let _ = msg.reply(ctx, "No such module.").await;
                            return true;
                        }
                    };

                    let _ = msg
                        .reply(
                            ctx,
                            format!(
                                "{item} is *{}*.",
                                if cmd.enabled { "enabled" } else { "disabled" }
                            ),
                        )
                        .await;
                }
                None => {
                    let module = match MasterSwitch::get(item) {
                        Some(module) => module,
                        None => {
                            let _ = msg.reply(ctx, "No such module.").await;
                            return true;
                        }
                    };

                    let mut commands = module.commands.iter().collect::<Vec<_>>();
                    commands.sort_by_key(|entry| entry.0);

                    let cmds =
                        commands
                            .iter()
                            .fold(String::new(), |mut current, (cmd, options)| {
                                write!(
                                    current,
                                    "\n\\- {}.{} is *{}*",
                                    item,
                                    cmd,
                                    if options.enabled {
                                        "enabled"
                                    } else {
                                        "disabled"
                                    }
                                )
                                .unwrap();
                                current
                            });

                    let _ = msg
                        .reply(
                            ctx,
                            format!(
                                "**[Module] {item}** is *{}* with {} commands.\n{cmds}",
                                if module.enabled {
                                    "enabled"
                                } else {
                                    "disabled"
                                },
                                commands.len()
                            ),
                        )
                        .await;
                }
            },
            _ => return false,
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
