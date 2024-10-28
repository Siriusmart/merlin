# Permissions

On every action, the permission system checks you against a "rule list" for that operation. Only if you passes the rule list the operation will continue, otherwise your message will be ignored or a rejection message will be given as response.

## Rule list

A rule list consist of multiple rules, for example

```js
1. +everyone
2. -dm
3. +@siriusmart
```

Rules closer to the bottom has higher priority than rules on top, you can think of it as rules later in the list will "override" previous rules. Each condition is prefixed with a `+` or `-` to show the rule is an "allow" or "disallow" type of rule.

In our example rule list, the checks runs in the sequence:
1. Check rule 3: Is your username "siriusmart"? If so, the check ends here and you are allowed to continue.
2. Check rule 2: Are you running this command in a dm? If so, the check ends here and you are not allowed to continue.
3. Check rule 1: Everyone is allowed, if you are not subjected to rule 2 or 3, then you are allowed to continue.

> If the user is subjected to none of the rules, then it is assumed that the user is allowed to continue.

## Syntax

Each rule starts with a modifier (`+` or `-`), followed by a condition.

```sh
[modifier][condition]
```

> Clearance presets is the only type of rule that doesn't begin with a modifier, this will be mentioned later.

### Conditions

Here's a list of all conditions

|Condition|Satisfied when|
|---|---|
|`everyone`|Always satisfied.|
|`everywhere`|Always satisfied.|
|`dm`|The command is ran in a DM to the bot.|
|`server`|The command is ran in a server.|
|`@user_name`|The user has the specified username.|
|`@user_id`|The user has the specified user ID.|
|`&server_id:role_name`|The user has the specified role.|
|`&server_id:role_id`|The user has the specified role of the ID.|
|`#channel_name`|The command is ran in a channel with the specified name.|
|`#channel_id`|The command is ran in a channel with the specified channel ID.|
|`%server_name`|The command is ran in a server with the specified name.|
|`%server_id`|The command is ran in a server with the specified ID.|

> Adding a role permissions in a server will automatically prepend it the rule with the server ID.

## Changing permissions

The command `core.perms` provides functionality for modifying module and command permissions.

```sh
$ .help perms

[Command] core.perms
Manage module permissions.

Usage:
.core perms [module] (rules...)
.core perms [module] clear
```

The argument `(rules...)` represent a list that is optional, for example

```sh
$ .perms core.ping -everyone +@siriusmart

Module permissions for core.ping updated. (not saved)
```

Disables `core.ping` for everyone except siriusmart. The rule `+@siriusmart` is later in the list, and therefore has a higher priority than `-everyone`.

### Viewing permissions

Permissions for existing commands can be viewed by not including a rule list, for example.

```js
$ .perms core.ping

[Permission] core.ping
1. -everyone
2. +@623823202073706496
```

> A command can be used only if it is allowed by *both* the rule lists for the command and for its parent module.
