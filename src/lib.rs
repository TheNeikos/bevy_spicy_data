#![deny(
    missing_docs,
    non_camel_case_types,
    non_snake_case,
    path_statements,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_allocation,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    while_true,
    array_into_iter,
    clippy::panic,
    clippy::print_stdout,
    clippy::todo,
    clippy::unwrap_used
)]
#![doc = include_str!("../README.MD")]

use bevy::{
    asset::{Asset, AssetLoader},
    prelude::*,
};
pub use bevy_spicy_data_derive::data_config;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

#[doc(hidden)]
pub mod private {
    pub use ::bevy::app::App;
    pub use ::bevy::asset::{AddAsset, LoadContext, LoadedAsset};
    pub use ::bevy::reflect::TypeUuid;
    pub use ::bevy::reflect::Uuid;
    pub use ::serde;
}

#[derive(Debug)]
/// Plugin for a given config struct, use the `Root` if you want to include all of the toml file
///
/// ## Examples
///
/// ```rust
/// # use bevy_spicy_data::data_config;
///
/// data_config!(pub config, "examples/simple.toml");
///
/// // Later in
///
/// ```
pub struct TomlConfigPlugin<T: Config> {
    kind: PhantomData<T>,
}

impl<T: Config + Sync + Send + 'static> Plugin for TomlConfigPlugin<T> {
    fn build(&self, app: &mut App) {
        T::add_asset(app);
        app.add_asset_loader(TomlAssetLoader::<T>::default());
    }
}

impl<T: Config> Default for TomlConfigPlugin<T> {
    fn default() -> Self {
        Self {
            kind: Default::default(),
        }
    }
}

/// The asset loader for the data you wish to load from a given file
///
/// You should not need to interact with it directly as the [`TomlConfigPlugin`] will
/// add it for you correctly.
#[derive(Debug)]
pub struct TomlAssetLoader<T: Config>(PhantomData<T>);

impl<T: Config> Default for TomlAssetLoader<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Config + Sync + Send + 'static> AssetLoader for TomlAssetLoader<T> {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let config: T = toml::from_slice(bytes)?;

            println!("loading new toml config");

            config.register(load_context, None);

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["config"]
    }
}

/// The principal trait for a piece of configuration
pub trait Config: DeserializeOwned + Asset {
    /// Register a piece of data at the given path.
    ///
    /// This allows you to only reference to a specific
    /// part of your configuration using bevy's subassets.
    fn register<'a>(
        &self,
        load_context: &'a mut bevy::asset::LoadContext,
        path: Option<Vec<String>>,
    );

    /// Register the given config piece as an asset
    fn add_asset(app: &mut bevy::app::App);
}
