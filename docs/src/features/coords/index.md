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
|`generic.private`|Entries added to this category will only be visible to the person who added the entry with no exception.|
|`[cogname].unspecified`|Entries added to `[cogname]` without specifying a subcategory will be added here, it has the same permission level as its parent category.|

## Operations

Each parent and subcategory can include a rule list. Users with permission to a category has full access to that category, a user have permission to a subcategory if it has permission to both the parent and child category.

### Access

A user with permission to a category can
- Edit category details.
- Edit entries within that category.

An *edit* action includes updating details, adding items and removing items.

### Clearance

- Users with the `?coorduser` clearance preset can use commands in the `coord` module.
- Users with the `?coordmod` clearance preset can edit permissions of categories.

> By default all new categories are created with the `?coordmod` preset, you will need to modify that to allow access to all users.
