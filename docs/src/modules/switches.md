# Switches

Each module and command can be enabled/disabled through *switching*.

The command `core.switch` provides functionality for enabling and disabling a module.

```sh
$ .help switch

[Command] core.switch
Enable/disable commands and modules.

Usage:
.core switch [module] (enable|disable)
```

### Module

The module `core` can be disabled with

```sh
$ .switch core disable

core has been disabled. (not saved)
```

### Command

> We will be considering commands to be a module, for example command `ping` in module `core` will have identifier `core.ping`.

The command `core.ping` can be disabled with

```sh
$ .switch core.ping disable

core.ping has been disabled. (not saved)
```

## Disable behaviour

A command can only be used if itself and its parent module are both enabled.

Disabling a module will take effect immediately, and the bot will not respond to any disabled commands. This change can be made persistent across reloads by writing any changed options to config using `.save`.

A disabled module may still appear in help pages, as there is no mechanics to unload modules at runtime. To fully unexist the module, a reload should be done using the `.reload` command. (make sure to run `.save` first!)
