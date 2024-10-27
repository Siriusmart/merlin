use std::fmt::Write;

use serenity::{
    all::{Context, Message},
    async_trait,
};

use crate::{sys::Command, Clearance, MasterSwitch, PerCommandConfig};

pub struct CmdPerms;

#[async_trait]
impl Command for CmdPerms {
    fn name(&self) -> &str {
        "perms"
    }

    fn description(&self) -> &str {
        "Manage module permissions."
    }

    fn usage(&self) -> &[&str] {
        &["[module] (rules...)", "[module] clear"]
    }

    async fn run(&self, args: &[&str], ctx: &Context, msg: &Message) -> bool {
        match args {
            [] => return false,
            [module] => {
                let (module, command) = if let Some((module, command)) = module.split_once('.') {
                    (module, Some(command))
                } else {
                    (*module, None)
                };

                if !MasterSwitch::has_module(module, command) {
                    let _ = msg.reply(ctx, "No such module.").await;
                    return true;
                }

                match command {
                    Some(cmd) => {
                        let percmd = MasterSwitch::get(module)
                            .unwrap()
                            .commands
                            .get(cmd)
                            .unwrap();

                        if percmd.allowed.is_empty() {
                            let _ = msg
                            .reply(
                                ctx,
                                format!(
                                    "**[Permission] {module}.{cmd}**\nThis module has no permission rules.",
                                ),
                            )
                            .await;
                            return true;
                        }

                        let _ = msg
                            .reply(
                                ctx,
                                format!(
                                    "**[Permission] {module}.{cmd}**{}",
                                    percmd.allowed.iter().enumerate().fold(
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
                    None => {
                        let permod = MasterSwitch::get(module).unwrap();

                        if permod.allowed.is_empty() {
                            let _ = msg
                            .reply(
                                ctx,
                                format!(
                                    "**[Permission] {module}**\nThis module has no permission rules.",
                                ),
                            )
                            .await;
                            return true;
                        }

                        let _ = msg
                            .reply(
                                ctx,
                                format!(
                                    "**[Permission] {module}**{}",
                                    permod.allowed.iter().enumerate().fold(
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
            }
            [module, "clear"] => {
                let (module, command) = if let Some((module, command)) = module.split_once('.') {
                    (module, Some(command))
                } else {
                    (*module, None)
                };

                if !MasterSwitch::has_module(module, command) {
                    let _ = msg.reply(ctx, "No such module.").await;
                    return true;
                }

                match command {
                    Some(cmd) => {
                        let percmd = MasterSwitch::get_mut(module)
                            .unwrap()
                            .commands
                            .get_mut(cmd)
                            .unwrap();

                        if percmd.allowed.is_empty() {
                            let _ = msg
                            .reply(
                                ctx,
                                format!(
                                    "Module permissions for **{module}.{cmd}** has been cleared, but is was originally empty.",
                                )
                            )
                            .await;
                            return true;
                        }

                        percmd.allowed.clear();

                        let _ = msg
                            .reply(
                                ctx,
                                format!(
                                    "Module permissions for **{module}.{cmd}** has been cleared. *(not saved)*",
                                )
                            )
                            .await;
                    }
                    None => {
                        let permod = MasterSwitch::get_mut(module).unwrap();

                        if permod.allowed.is_empty() {
                            let _ = msg
                            .reply(
                                ctx,
                                format!(
                                    "Module permissions for **{module}** has been cleared, but is was originally empty.",
                                )
                            )
                            .await;
                            return true;
                        }

                        permod.allowed.clear();

                        let _ = msg
                            .reply(
                                ctx,
                                format!(
                                    "Module permissions for **{module}** has been cleared. *(not saved)*",
                                    )
                            )
                            .await;
                    }
                }
            }
            [module, ..] => {
                let (module, command) = if let Some((module, command)) = module.split_once('.') {
                    (module, Some(command))
                } else {
                    (*module, None)
                };

                if !MasterSwitch::has_module(module, command) {
                    let _ = msg.reply(ctx, "No such module.").await;
                    return true;
                }

                if !Clearance::validate(&args[1..], true) {
                    let _ = msg
                        .reply(
                            ctx,
                            format!("Failed to update module permission for {} because it contains invalid rules.",args[0]),
                        )
                        .await;
                    return true;
                }

                let mut allowed = args[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>();

                if !Clearance::map_rules(&mut allowed, msg, ctx).await {
                    return true;
                }

                match command {
                    Some(cmd) => {
                        let percmd = MasterSwitch::get_mut(module)
                            .unwrap()
                            .commands
                            .get_mut(cmd)
                            .unwrap();

                        percmd.allowed = allowed;

                        let _ = msg
                            .reply(
                                ctx,
                                format!(

                                "Module permissions for **{module}.{cmd}** updated. *(not saved)*",
                                    ),
                            )
                            .await;
                    }
                    None => {
                        let permod = MasterSwitch::get_mut(module).unwrap();

                        permod.allowed = allowed;

                        let _ = msg
                            .reply(
                                ctx,
                                format!(
                                    "Module permissions for **{module}** updated. *(not saved)*",
                                ),
                            )
                            .await;
                    }
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
