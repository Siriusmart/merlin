# [Command] Coords.Attach

Attach files to an entry.

## Upload by name

You may upload any number of files to Merlin.

```sh
.attach big-base [attachments]
```

This command only succeed if there is a visible entry with name `big-base`.

> If the message does not include any attachments, the command will not succeed.

## Filters

You may use search filters as specified in [`find`](./find.md).

```sh
.attach field1=value1 field2=value2 [attachments]
```

> If more than one entry match the filters, the command will not succeed.

## Permissions

- Anyone can attach entries in `generic.unspecified` and their own entries in `generic.private`.
- Users can attach entries in a category if they have access to it.
