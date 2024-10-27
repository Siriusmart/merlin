# [Command] Coords.CogAdd

Create a new category.

## Categories

Entries are organised into categories and subcategories, all entries *must* be under a category and subcategory.

### Creating a category

```sh
.cogadd new-category # where `new-category` is the name of the category
```

This command will only succeed if there isn't another category with the name `new-category`.


### Creating a subcategory

```sh
.cogadd new-category.new-subcategory # where `new_subcategory` is the name of the subcategory
```

This command will only succeed if the parent category `new-category` exists, and there isn't another subcategory with the name `new-subcategory` under the same parent category.

### Category description

A category (or subcategory) can be created with a description by passing it as an additional argument.
```sh
.cogadd new-category 'This is a description.'
.cogadd new-category.new-subcategory 'This is another description.'
```

> The single quotes shows that it is a single argument.

## Permissions

- Anyone can create top level categories.
- Users can only create subcategories under a parent category if they have access to the parent category.
- New subcategories cannot be created under the `generic` parent category.
