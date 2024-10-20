# Merlin

Enchanter for the roundtable.

## Running the bot

1. Install Merlin
```
cargo install --git https://github.com/siriusmart/merlin
```
2. Running for the first time will generate config files at `~/.config/merlin/`, put you bot token there.
3. Save the config files and run it.

## Configuration

Config files are autogenerated in ~/.config/merlin/, format is jsonc - standard json that allows comments.
- master.jsonc should be self explanatory
- switch.jsonc contains permission configuration for each module and command, missing entries are automatically added. The "allowed" field specifies who is allowed to use the command/module. A user can use a command only if he has permission to use *both* the command and the module. The later the rule is on the list, the higher its priority is. If no rule matches, it is assumed that the user is allowed to use the command. Here's a list of rules
    - `+[conditon]` allows the user to use the command if condition meets, `-[condition]` does the opposite.
    - `+#channel name` or `+#[channelid]` allows the command to be used in a channel.
    - `+@user name` or `+@[userid]` allow the command to be ran be the user.
    - `+&role name` or `+&[roleid]` allows the command to be ran by a role, note that using the role ID is preferred over role name due to performance issues.
    - `+dm`, `+server`, `+everyone`, `+everywhere` are special conditions which name explains itself.

TODO:
- Coords tags
- Coords restore action
- Command logging

Issue tracker:
- Role based permissions not working outside the server
- Coord entries should check for nearby when being edited, and disallow bulk edit to a new location.
