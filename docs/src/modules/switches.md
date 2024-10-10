# Switches

Each module and command can be enabled/disabled through *switching*.

> We will be considering commands to be a module, for example command `ping` in module `core` will have identifier `core.ping`.

The command `core.switch` provides functionality of enabling and disabling a module.

```sh
$ .help switch

[Command] core.switch
Enable/disable commands and modules.

Usage:
.core switch [module] (enable|disable)
```

For example, the command `core.ping` can be disabled with

```sh
$ .switch core.ping disable

core.ping has been disabled. (not saved)
```
