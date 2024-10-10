# Installation

This page assumes you have Rust and Cargo installed. [*Here's*](https://www.rust-lang.org/tools/install) a guide on how to get them.

## Install from Git

You can install the latest commit from Git with Cargo.

```sh
cargo install --git https://github.com/siriusmart/merlin
```

The installed binary can be found in `~/.cargo/bin/`.

> Merlin does not auto update, for updates you will have to watch the repository.

## Features

By default Merlin compiles with all features. If there are features you don't need or wish to be replaced, you can compile without them using the `--no-default-features` flag.

Features can be added to a *featureless* install with the `-F [feature]` feature flag, the installation command can include as many feature flags as needed.

Here's a list of available features:

|Feature|Description|
|--|--|
|`modcore`|Core module, provide basic functionalities.|
|`modcoords`|Coords DB module for anarchy servers, WIP.|
