use std::{collections::HashMap, sync::OnceLock};

use async_recursion::async_recursion;
use serenity::{
    all::{Context, Message},
    Client,
};

use super::{Config, MasterSwitch, Module, MASTER};

static HANDLER: OnceLock<CommandHandler> = OnceLock::new();

pub struct CommandHandler {
    pub modules: HashMap<String, Box<dyn Module>>,
    pub alias: HashMap<String, String>,
}

impl CommandHandler {
    fn new() -> CommandHandler {
        Self {
            modules: Default::default(),
            alias: Default::default(),
        }
    }

    pub fn add_module<M: Module + 'static>(&mut self, module: M) {
        self.modules
            .insert(module.name().to_string(), Box::new(module));
    }

    #[async_recursion]
    pub async fn run(args: &[&str], ctx: &Context, msg: &Message) {
        if !args.is_empty() {
            if args[0] == "help" {
                Self::help(&args[1..], &ctx, &msg).await;
                return;
            }

            let handler = HANDLER.get().unwrap();

            if let Some(module) = handler.modules.get(args[0]) {
                module.run(&args[1..], &ctx, &msg).await;
            } else if let Some(alias) = handler.alias.get(args[0]) {
                CommandHandler::run(
                    [
                        shell_words::split(alias)
                            .unwrap()
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .as_ref(),
                        &args[1..],
                    ]
                    .concat()
                    .as_ref(),
                    ctx,
                    msg,
                )
                .await;
            }
        }
    }

    #[async_recursion]
    async fn help(args: &[&str], ctx: &Context, msg: &Message) {
        let handler = HANDLER.get().unwrap();

        match args {
            [module_str, command_str, ..]
                if handler.modules.contains_key(*module_str)
                    && handler
                        .modules
                        .get(*module_str)
                        .unwrap()
                        .commands()
                        .contains_key(*command_str) =>
            {
                let module = handler.modules.get(*module_str).unwrap();
                let all_commands = module.commands();
                let command = if let Some(cmd) = all_commands.get(*command_str) {
                    cmd
                } else {
                    Self::help(&[module_str], ctx, msg).await;
                    return;
                };

                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "**[Command] {}.{}**\n{}\n\n**Usage**: {}{} {}",
                            module.name(),
                            command.name(),
                            command.description(),
                            MASTER.get().unwrap().prefix,
                            command.name(),
                            command.usage()
                        ),
                    )
                    .await;
            }
            [module, ..] if handler.modules.contains_key(*module) => {
                let module = handler.modules.get(*module).unwrap();
                let _ = msg
                    .reply(
                        ctx,
                        format!(
                            "**[Module] {}**\n{}{}{}",
                            module.name(),
                            module.description(),
                            if module.commands().is_empty() {
                                "".to_string()
                            } else {
                                let mut commands = module
                                    .commands()
                                    .keys()
                                    .map(|label| format!("\\- {}", label))
                                    .collect::<Vec<_>>();
                                commands.sort();
                                format!("\n\n**Commands**\n{}", commands.join("\n"))
                            },
                            if module.aliases().is_empty() {
                                "".to_string()
                            } else {
                                let mut aliases = module
                                    .aliases()
                                    .iter()
                                    .map(|(from, to)| format!("\\- {from} → {to}"))
                                    .collect::<Vec<_>>();
                                aliases.sort();
                                format!("\n\n**Aliases**\n{}", aliases.join("\n"))
                            }
                        ),
                    )
                    .await;
            }
            [alias] if handler.alias.contains_key(*alias) => {
                Self::help(
                    shell_words::split(handler.alias.get(*alias).unwrap())
                        .unwrap()
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .as_ref(),
                    ctx,
                    msg,
                )
                .await
            }
            _ => {
                let _ = msg
                    .reply(
                        ctx,
                        format!("**Available modules**\n{}", {
                            let mut modules = handler
                                .modules
                                .keys()
                                .map(|label| format!("\\- {}", label))
                                .collect::<Vec<_>>();
                            modules.sort();
                            modules.join("\n")
                        }),
                    )
                    .await;
            }
        }
    }

    pub async fn load(client: &Client, switch: &mut MasterSwitch) {
        let mut switch_modified = false;

        let mut handler = Self::new();
        handler.register();

        for module in handler.modules.values_mut() {
            match switch.0.get_mut(module.name()) {
                Some(permod) => {
                    for (cmd_label, cmd) in module.commands().iter() {
                        if !permod.commands.contains_key(cmd_label) {
                            switch_modified = true;
                            permod.commands.insert(cmd_label.to_string(), cmd.percmd());
                        }
                    }

                    if !permod.enabled {
                        continue;
                    }
                }
                None => {
                    switch_modified = true;
                    switch.0.insert(module.name().to_string(), module.permod());
                }
            }

            module.setup(client).await;

            for (from, to) in module.aliases() {
                handler.alias.insert(from.to_string(), to.to_string());
            }
        }

        if switch_modified {
            switch.save();
        }

        let _ = HANDLER.set(handler);
    }
}
