# Clearance presets

Clearance presets are macros that can be used in rule lists.

Using presets allow for changes in permission to propagate to multiple rule lists using the preset. Presets can be used in rule lists with the `?preset_name` syntax.

## Example

Suppose we have the preset "admin" as follows

```js
1. +&bot_masters
2. +@siriusmart
3. +#admin_chat
```

This preset can be used in other rule lists

```js
1. -everyone
2. ?admin
3. -dm
```
since preset work the same as macros, the rule list above is equivalent to

```js
1. -everyone
2. +&bot_masters
3. +@siriusmart
4. +#admin_chat
5. -dm
```
where the `?admin` rule is replaced by contents in the preset.

> Using a preset that doesn't exist is the same as using an empty preset, this is unsounding so make sure you are not making a type when specifying presets.

## Managing presets

The command `core.clearance` provides functionality for modifying presets.

```sh
$ .help clearance

[Command] core.clearance
Manage clearance presets.

Usage:
.core clearance (preset)
.core clearance [preset] (rules...)
.core clearance [preset] clear
```

Using the command without arguments responds with a list of all nonempty presets. Preset content can be changed similar to how permission is changed.

```js
$ .clearance admin +&bot_masters +@siriusmart +#admin_chat

Clearance preset admin updated.
```

> Note that circular definitions of presets are not allowed.
