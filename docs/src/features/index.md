# Features

Features are essentially a module system at compile time - a feature can be disabled at compile such that it is physically not in the bot. (Ideally) features are independent of each other, and can be developed separately.

The bot currently includes 2 features, both are enabled by default.
- modcore
- modcoords

To only compile certain features, use the cargo compile flags

```sh
$ cargo build --release --no-default-features -F feature1 -F feature2 # ...
```
where `feature1` and `feature2` are features you wish to compile.

## Modules

Modules are loaded in when the bot starts. Each module consists of

- A help page
- Multiple commands
- Multiple aliases
- A default command

### Aliases

Aliases are decoy commands that redirects you to another command. For example, given the alias `ping → core ping` running

```sh
$ .ping
```
will be recognised as running
```sh
$ .core ping
```
where `ping` is replaced by `core ping`.

### Default command

A default command is essentially an alias, when calling a module without specifying a command, the "default command" will get ran instead. For example, `core → help core`. Which is why when running `.core` you will see the help page.

## Module documentation

The following pages will contain information about various modules and their commands.
