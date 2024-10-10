# Modules

Modules are independent units with a specific function - enabling/disabling one module will have no effect on one another. Modules that are disabled will not be loaded on startup.

## Structure

Each module is a collection of commands and aliases, it also provides a brief description of itself to be used in the help commands.

Each module contains the following information:
```yml
name: core
description: Core service modules.

aliases:
    - ping → core ping
    - version → core version

commands:
    - ping
    - version
```

And each command contains the following information:
```yml
name: ping
description: Check shard network latency.

usage:
    - .core ping
```

## Module options

Module/command can be enabled and disabled without restarting the bot, and permission rules can be applied, which will be discussed in greater detail in following chapters.

> Bot options can only be modified by `admins`, you can give yourself permission to edit options by adding `+@[user_id]` to `~/.config/merlin/clearance.jsonc`.
