//! An example game written in the Bevy game engine and using the [`agb`] crate to allow running it
//! on the Game Boy Advance.

//! We declare our crate as `no_std`, as the Game Boy Advance doesn't have a port of the standard
//! library.
#![no_std]

//! We also declare the crate as not having a typical `main` function.
//! The `agb-gbafix` tool we use to generate our final ROM file expects an exported
//! function named `main` accepting no arguments and _never_ returning.
//! This is handled by [`main`].
#![no_main]

//! [`agb`] provides a global allocator, allowing us to use items from the [`alloc`] crate.
extern crate alloc;

use log::info;
use mel0n::{
    agb,
    bevy::{
        app::PanicHandlerPlugin,
        diagnostic::{DiagnosticsPlugin, FrameCountPlugin},
        input::{
            gamepad::{gamepad_connection_system, gamepad_event_processing_system},
            InputSystem,
        },
        prelude::*,
        state::app::StatesPlugin,
        time::TimePlugin,
    },
    bevy_mod_gba::{prelude::*, AgbSoundPlugin, Sprite},
    fruit::add_fruit,
    gba::{Mel0nGbaPlugin, Mel0nGbaSetupSet},
    wall::add_walls,
    Mel0nBasePlugin,
};

/// Main entry point.
#[expect(unsafe_code)]
#[unsafe(export_name = "main")]
pub extern "C" fn main() {
    // We can use Bevy's `App` abstraction just like any other Bevy application.
    let mut app = App::new();

    // The first step is to add the `AgbPlugin`.
    // This sets up integration between Bevy and the `agb` abstraction over the GameBoy Advance.
    // This _must_ be done first, as it also sets up `Instant` for us.
    // Otherwise, the `TimePlugin` will fail to initialize.
    app.add_plugins(AgbPlugin.set(AgbSoundPlugin {
        enable_dmg: true,
        ..default()
    }));

    // Next we can add any Bevy plugins we like.
    // TODO: Used `DefaultPlugins` instead of this explicit list.
    // `DefaultPlugins` includes `InputPlugin` which is problematic on the GameBoy Advance. See below.
    app.add_plugins((
        PanicHandlerPlugin,
        TaskPoolPlugin::default(),
        FrameCountPlugin,
        TimePlugin,
        TransformPlugin,
        DiagnosticsPlugin,
        StatesPlugin,
    ));

    // TODO: Type registration information from `InputPlugin` causes an OOM error.
    // So we manually register the parts of this plugin that we need and ignore the rest.
    app.add_systems(
        PreUpdate,
        (
            gamepad_connection_system,
            gamepad_event_processing_system.after(gamepad_connection_system),
        )
            .in_set(InputSystem),
    );

    // Unfortunately, we currently don't have a first-party abstraction for assets or rendering.
    // This means getting assets, and rendering them must be done somewhat manually.
    app.add_plugins(Mel0nGbaPlugin);

    app.add_plugins(Mel0nBasePlugin);
    app.insert_resource(Time::<Virtual>::from_max_delta(
        core::time::Duration::from_secs(5),
    ));

    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.run();

    agb::syscall::stop();
}
