# [Command] Coords.CogEdit

Edit an existing category.

## Category details

Each category contains the following fields.

|Field|Description|
|---|---|
|`name`|A unique identifier.|
|`desc`|An optional description text.|

### Renaming category
```sh
.cogedit category name=new-name
```

This command renames the category with name `category` to `new-name`, provided a category with name `category` exists.

### Renaming subcategory
```sh
.cogedit category.subcategory name=new-name
```

This command renames `category.subcategory` to `category.new-name`, you cannot change a subcategory's parent by renaming it.

### Changing description

The text description of a category or subcategory can be edited.

```sh
.cogedit category desc='This is a description.'
.cogedit category.subcategory desc='This is a description.'
```

### Removing description

An empty description text is treated not exist.

```sh
.cogedit category desc=''
```
Where `''` shows that there is an empty argument following `desc=`.

## Permissions

- Users can only edit a parent category if they have access to it.
- Users can only edit a child category if they have access to both the parent and child category.
- `generic.*` and `*.unspecified` are system categories that cannot be edited.
