# [Command] Coords.Find

Search for entries.

## Simple search

### Search for a single entry

Search for an entry using its name.

```sh
.find big-base
```

This command only succeed if there is a visible entry with name `big-base`.

### Search for a category

List entries in a category.

```sh
.find category
.find category.subcategory
```

This command only succeed if the specified category exists.

## Filters

The following filters can be applied when searching.

|Field|Description|Format|
|---|---|---|
|`dim`|Dimension of the location, allowes `ow` (overworld), `nether` and `end`.|`[Dimension]`|
|`cog`|Category of the entry.|`[Category]` or `[Category].[Subcategory]`|
|`near`|Search for entries at a radius from a specific point.|`[x],[z],[radius]`|
|`tags`|Search for entries containing all of the specified tags.|`[Tag],[Tag],..`|
|`page`|If the filter allows for a large number of entries, you may specify a page number. (default=1)|`[Integer]`|

### Usage

```sh
.find field1=value1 field2=value2

# search for entries in the overworld and within 5000 blocks of spawn
.find dim=ow near=0,0,5000

# search page for of entries under the `farm` category, with tags `afkable` and `exp`
.find cog=farm tags=afkable,exp page=3
```

You may also list out all visible entries with
```sh
.find *
```

## Permissions

- Anyone can view entries from `generic.unspecified` and their own entries in `generic.private`.
- Users can view entries from a category if they have access to it.
