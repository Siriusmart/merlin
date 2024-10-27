# [Command] Coords.CogPerms

Edit permissions to a category.

## Usage

This command uses the same rules for permission as [`core.perms`](../../modules/permissions.md).

### Setting permissions

```sh
.cogperms category [rule list]
.cogperms category.subcategory [rule list]
```

Read more about rule lists [here](../../modules/permissions.html#rule-list).

### Removing permissions

```sh
.cogperms category clear
.cogperms category.subcategory clear
```

## Permissions

- Anyone allowed in the `?coordmod` clearance preset (customisable).
