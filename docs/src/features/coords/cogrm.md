# [Command] Coords.CogRm

Removes an existing category.

## Caveat

This command can only be used to remove empty categories.

### Removing a category

```sh
.cogrm category
.cogrm category.subcategory
```

## Permissions

- Users can only remove a parent category if they have access to it.
- Users can only remove a child category if they have access to both the parent and child category.
- `generic.*` and `*.unspecified` are system categories that cannot be removed.
