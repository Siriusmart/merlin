# [Command] Coords.CoordEdit

Bulk edit existing entries.

## Entry details

All fields of an entry can be edited.

|Field|Description|Format|
|---|---|---|
|`newname`|Unique name for the entry.|`[String]`|
|`newdesc`|Description text for the entry.|`[String]`|
|`newdim`|Dimension of the location, allows `ow` (overworld), `nether` and `end`.|`[String]`|
|`newpos`|Whole number coordinates in format of `x,z`.|`[int],[int]`|
|`newcog`|Category to move to.|`[String]` or `[String].[String]`|
|`newtags`|List of tags.|`[String],[String],...`|

### Edit a single entry

A single entry can be specified by its unique name.

```sh
.coordedit [entry name] field1=value1 field2=value2

# moves entry to the `base.member` category
.coordedit big-base newcog=base.member

# updates description to `A very cool base` and set tags to `meeting` and `stash`
.coordedit big-base newdesc='A very cool base.' newtags=meeting,stash
```

### Bulk editing

You may apply search filters as specified in [`find`](./find.md#filters) to edit entries in bulk.

```sh
# moves all visible entries within a 100 block radius to 10000 10000 to the `mainbase.private` category
.coordedit near=10000,10000,100 newcog=mainbase.private
```

> Note that you cannot move an entry not created by you to `generic.private`.

## Permissions

- Anyone can edit entries from `generic.unspecified` and their own entries in `generic.private`.
- Users can edit entries from a category if they have access to it.
