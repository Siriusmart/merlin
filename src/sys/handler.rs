use std::{collections::HashMap, fmt::Write, sync::OnceLock};

use async_recursion::async_recursion;
use serenity::{
    all::{Context, Message},
    Client,
};

use super::{Config, MasterSwitch, Module, MASTER};

static mut CLIENT: OnceLock<Client> = OnceLock::new();
static mut HANDLER: OnceLock<CommandHandler> = OnceLock::new();

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

    pub fn client_mut() -> &'static mut Client {
        unsafe { CLIENT.get_mut() }.unwrap()
    }

    pub fn client() -> &'static Client {
        unsafe { CLIENT.get() }.unwrap()
    }

    pub fn client_set(client: Client) {
        let _ = unsafe { CLIENT.set(client) };
    }

    pub fn add_module<M: Module + 'static>(&mut self, module: M) {
        self.modules
            .insert(module.name().to_string(), Box::new(module));
    }

    #[async_recursion]
    pub async fn run(args: &[&str], ctx: &Context, msg: &Message) {
        if !args.is_empty() {
            if args[0] == "help" {
                if !args.is_empty()
                    && MasterSwitch::get(args[0]).is_some_and(|permod| !permod.enabled)
                {
                    return;
                }

                Self::help(&args[1..], ctx, msg).await;
                return;
            }

            if MasterSwitch::get(args[0]).is_some_and(|permod| !permod.enabled) {
                return;
            }

            let handler = unsafe { HANDLER.get() }.unwrap();

            if let Some(module) = handler.modules.get(args[0]) {
                module.run(&args[1..], ctx, msg).await;
            } else {
                for i in 0..args.len() {
                    if let Some(alias) = handler.alias.get(
                        &args
                            .iter()
                            .take(i + 1)
                            .copied()
                            .collect::<Vec<_>>()
                            .join(" "),
                    ) {
                        CommandHandler::run(
                            [
                                shell_words::split(alias)
                                    .unwrap()
                                    .iter()
                                    .map(String::as_str)
                                    .collect::<Vec<_>>()
                                    .as_ref(),
                                &args[i + 1..],
                            ]
                            .concat()
                            .as_ref(),
                            ctx,
                            msg,
                        )
                        .await;
                        break;
                    }
                }
            }
        }
    }

    #[async_recursion]
    pub async fn help(args: &[&str], ctx: &Context, msg: &Message) {
        let handler = unsafe { HANDLER.get() }.unwrap();

        let args = args
            .iter()
            .flat_map(|arg| arg.split('.'))
            .collect::<Vec<_>>();

        match args.as_slice() {
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
                            "**[Command] {}.{}**\n{}\n\n**Usage**:{}",
                            module.name(),
                            command.name(),
                            command.description(),
                            match command.usage() {
                                [] => format!(
                                    "\n{}{} {}",
                                    unsafe { MASTER.get() }.unwrap().prefix,
                                    module.name(),
                                    command.name()
                                ),
                                any => any.iter().fold(String::new(), |mut current, usage| {
                                    write!(
                                        current,
                                        "\n{}{} {} {}",
                                        unsafe { MASTER.get() }.unwrap().prefix,
                                        module.name(),
                                        command.name(),
                                        usage
                                    )
                                    .unwrap();
                                    current
                                }),
                            }
                        ),
                    )
                    .await;
            }
            [module, ..] if handler.modules.contains_key(*module) => {
                let module = handler.modules.get(*module).unwrap();
                let switch = MasterSwitch::get(module.name()).unwrap();

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
                                    .filter(|k| switch.commands.get(k.as_str()).unwrap().enabled)
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
                                .filter(|k| MasterSwitch::get(k).unwrap().enabled)
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

    pub async fn load(reload: bool) {
        let switch = MasterSwitch::get_mut_self();
        let mut switch_modified = false;

        let mut handler = Self::new();
        handler.register();

        let mut disabled_modules = Vec::new();

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
                        disabled_modules.push(module.name().to_string());
                        continue;
                    }
                }
                None => {
                    switch_modified = true;
                    switch.0.insert(module.name().to_string(), module.permod());
                }
            }

            if reload {
                module.reload().await
            } else {
                module.setup().await;
            }

            for (from, to) in module.aliases() {
                handler.alias.insert(from.to_string(), to.to_string());
            }
        }

        if switch_modified {
            switch.save();
        }

        for disabled in disabled_modules {
            handler.modules.remove(&disabled).unwrap();
        }

        let _ = unsafe { HANDLER.set(handler) };
    }

    pub async fn reload() {
        unsafe { HANDLER = OnceLock::new() };
        Self::load(true).await;
    }
}
