# [Module] Coords DB

Searchable coordinates for anarchy servers.

## Categorisation

Each entry is associated with a parent category and a child category, similar to the *binary nomenclature* used in naming species.

For example, `farm.enderman` would be a valid name for a category.

### Reserved categories

Most categories can be treated as a folder where privileged users can access. However, there are some reserved categories with special behaviours.

|Category|Description|
|---|---|
|`generic.unspecified`|When an entry is added without specifying a category, it will be assumed to be in this category. When deleting a main category, its entries will be automatically moved here.|
|~~`generic.private`~~|Entries added to this category will only be visible to the person who added the entry with no exception.|
|`[cogname].unspecified`|Entries added to `[cogname]` without specifying a subcategory will be added here, it has the same permission level as its parent category.|

> `generic.private` is basically unusable because role based permissions are currently not working in DMs.

## Operations

Each parent and subcategory can include a rule list. Users with permission to a category has full access to that category, a user have permission to a subcategory if it has permission to both the parent and child category.

> By default, all commands have requires the `?coorduser` clearance.

### Add a category

Main article: `cogadd`

Parent categories can be created by anyone with access to the command, child categories can only be created with users with access to the parent category.

> You cannot create subcategories for the `generic` category.

### Add an entry

Main article: `coordadd`

Anyone with access to a category can add entries to it. Since `general.unspecified` can be access by anyone with permission to the command, users can always add entries to it.

> Entries will not be added if there already exist an entry within close proximity of the new entry.

### Edit a category

Main article: `cogedit`

The name and description of a category can be modified by anyone with access to that category.

### Edit an entry

Main article: `coordedit`

Name, description and category of multiple entries can be bulk edited with a single command.

### Find an entry

Main article: `find`

Find entries by its name, category, and distance from a location in world.

### Remove a category

Main article: `cogrm`

Remove a category, all items within that category will be moved to `parent.unspecified` is a subcategory is deleted, or `generic.unspecified` if the parent category is deleted, note that permissions required to view an entry may change as a result.

### Remove an entry

Main article `coordrm`

Remove an entry, full stop.

### Manage category permissions

Main article: `cogperms`

Manage permissions required to access each category.

> This command requires the `?coordmod` clearance by default.

### ~~Revert an action~~

WIP: provides an ID for each action so that the reverse can be ran if something is done accidentally.
