# Basic usage

A list of commands can be accessed through the `help` command.

## Command structure

A command can be called by specifying its full name. For example
```sh
.core ping
```

Calls the `ping` command in the `core` module.

## The help command

The help command is an internal command (does not belong to any module, cannot be disabled) to display information about a module or command.

### Module

A module is a collection of command of similar function, for example the `core` module provides commands for basic housekeeping. Whereas the `coords` module contains everything to do with the coords DB feature.

```sh
$ .help core

[Module] core
Core service modules.

Commands
- ping
- uptime
- version

Aliases
- ping → core ping
- uptime → core uptime
- version → core version
```

### Alias

An alias is a shortcut for another command, in the example above, the command `.ping` will be aliased to `.core ping`, so a command can be ran without mentioning its module.

### Command

A command *does something*. Commands can be disabled globally, and per-user permission can be set up. When running a command that is disabled or without sufficient permission, it will be ignored by the bot.

```sh
$ .help core.ping

[Command] core.ping
Check shard network latency.

Usage:
.core ping
```

> You can also use alias on help: `.help ping` is equivalent to `.help core.ping`.

### Usage

Command usage can be displayed by using `.help` on the command. Each line in usage example represents a different way of using the command.

```
Usage:
.module command [required argument] (optional argument) (remaning arguments...)
.module command clear (either this|or this)
```
