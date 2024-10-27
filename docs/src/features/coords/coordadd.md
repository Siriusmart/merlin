# [Command] Coords.CoordAdd

Add an entry.

## Entries

Each entry corresponds to a location in the Minecraft world, when creating an entry, the following details must be provided.

|Field|Description|
|---|---|
|`name`|Unique name for the entry.|
|`dim`|Dimension of the location, allowes `ow` (overworld), `nether` and `end`.|
|`x`|Whole number X coordinate.|
|`z`|Whole number Z coordinate.|

### Adding an entry (minimum)

Providing only the required details, details of the entry can be edited separately.

```sh
.coordadd [name] [dim] [x] [z]
.coordadd big-base ow 1234 -5678
```

> Since a category is not specified, the entry will be added to `generic.unspecified`.

### Adding an entry with additional info

Full details can be provided on creation, detailed description of each field can be found in `coords.coordedit`.

```
.coordadd [name] [dim] [x] [z] (category)
.coordadd [name] [dim] [x] [z] (category) (description)
.coordadd [name] [dim] [x] [z] (category) (description) (tags)
```

Where tags is a list of comma separated values with no whitespace in between each item. For example `tag1,tag2,tag3`.

> The user will be notified to edit an existing entry instead if there exist a visible entry within close proximity of the new entry (customisable).

## Permissions

- Anyone can add entries to `generic.unspecified` and `generic.private`.
- Users can add entries to a category if they have access to it.
- Entries added to `generic.private` is only visible to the author.
