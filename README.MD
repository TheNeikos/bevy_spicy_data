# Bevy Compiletime Toml Integration

This crate allows you to have a compile time integration of your configuration files, and live-reloading while you are developing!

Check the examples folder on how to use this crate.

## Overview

`bevy_spicy_data` works in two steps:

- First, you define which file should be analyzed and included into your project with the `data_config!` macro.
- Second, you then register it as a plugin into your bevy game: `app.add_plugin(bevy_spicy_data::TomlConfigPlugin::<config::Root>::default())`

An invocation of the macro like this: `data_config!(pub config, "assets/game.config")` expands to roughly this representation:

```toml
# Contents of assets/game.config
["display"]
text = "Hello Bevy, reloaded! \n Heya"
```

```rust,ignore
pub mod config {
    pub mod display {
        pub struct Text(String);
    }

    pub struct Display {
        text: display::Text,
    }

    pub struct Root {
        display: Display,
    }
}
```

Each table becomes its own struct, containing whatever fields you have defined.
You can also nest tables.

This fits well together with bevy's asset system, as each 'file' can create multiple sub-assets
one can precisely define which part of the file one is interested in at each moment.

For example, if one were to load the whole file, the asset result would be `config::Root`.
If one would just want the data in the `text` field, then one can load just the sub-path `"game.config#display.text"`.

Check the "text" example on how this looks like in live.

