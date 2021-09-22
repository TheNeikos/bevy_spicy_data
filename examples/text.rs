use bevy::prelude::*;

use bevy_spicy_data::{UiDataText, data_config};

data_config!(pub config, "assets/game.config");

/// This example illustrates how to create UI text and update it in a system. It displays the
/// current FPS in the top left corner, as well as text that changes colour in the bottom right.
/// For text within a scene, please see the text2d example.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_spicy_data::TomlConfigPlugin::<config::Root>::default())
        .add_startup_system(setup)
        .add_system(text_update_system)
        .run();
}
// A unit struct to help identify the color-changing Text component
struct TomlText(Handle<config::display::Text>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    let toml_asset_handle = asset_server.load("game.config#display some.text");
    let text_handle = asset_server.load("game.config#system.debug");
    // UI camera
    commands.spawn_bundle(UiCameraBundle::default());
    // Text with one section
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(0.5),
                    right: Val::Percent(0.5),
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "hello\nbevy!",
                TextStyle {
                    font: asset_server.load("Share-Regular.ttf"),
                    font_size: 100.0,
                    color: Color::WHITE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .insert(TomlText(toml_asset_handle));

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(0.5),
                    left: Val::Percent(0.5),
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "{placeholder}",
                TextStyle {
                    font: asset_server.load("Share-Regular.ttf"),
                    font_size: 100.0,
                    color: Color::MIDNIGHT_BLUE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .insert(UiDataText::<config::system::Debug>(text_handle));
}

fn text_update_system(
    mut toml_text_events: EventReader<AssetEvent<config::display::Text>>,
    toml_text_assets: Res<Assets<config::display::Text>>,
    mut query: Query<(&mut Text, &TomlText)>,
) {
    for _event in toml_text_events.iter() {
        info!("New event: {:?}", _event);
        for (mut text, toml_text) in query.iter_mut() {
            if let Some(toml_text) = toml_text_assets.get(&toml_text.0) {
                text.sections[0].value = toml_text.0.clone();
            }
        }
    }
}
