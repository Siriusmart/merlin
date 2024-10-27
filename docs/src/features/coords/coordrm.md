# [Command] Coords.CoordRm

Remove an entry from coord DB.

## Remove a single entry

Remove an entry using its unique name.

```sh
.coordrm big-base
```

This command only succeed if there is a visible entry with name `big-base`.

## Bulk remove

You may use search filters as specified in [`find`](./find.md).

```sh
.coordrm field1=value1 field2=value2
```

## Permissions

- Anyone can remove entries from `generic.unspecified` and their own entries in `generic.private`.
- Users can remove entries from a category if they have access to it.
