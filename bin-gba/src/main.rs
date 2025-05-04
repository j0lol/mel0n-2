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

use bevy::{
    app::PanicHandlerPlugin,
    diagnostic::{DiagnosticsPlugin, FrameCountPlugin},
    input::{
        gamepad::{gamepad_connection_system, gamepad_event_processing_system},
        InputSystem,
    },
    prelude::*,
    state::app::StatesPlugin,
    time::TimePlugin,
};
use bevy_mod_gba::{prelude::*, AgbSoundPlugin, Sprite};
use log::info;
use mel0n::{
    fruit::add_fruit,
    gba::{Mel0nGbaPlugin, Mel0nGbaSetupSet},
    physics::{apply_collisions, apply_friction, apply_gravity, apply_velocity},
    wall::add_walls,
    Mel0nBasePlugin
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

    // This is our game logic and is entirely independent of the platform we're targeting.
    // app.add_systems(Startup, (add_fruit, add_walls).after(Mel0nGbaSetupSet))
    //     // .add_systems(Update, log_player_position)
    //     .add_systems(
    //         FixedUpdate,
    //         (
    //             // control_player,
    //             // flip_player_sprite,
    //             apply_gravity,
    //             apply_friction,
    //             apply_velocity,
    //             apply_collisions,
    //             // clamp_player_to_screen,
    //             // reset_jumps,
    //         )
    //             .chain(),
    //     );
    app.add_plugins(Mel0nBasePlugin);

    app.run();

    agb::syscall::stop();
}

// fn log_player_position(player: Single<&Transform, With<Player>>) {
//     info!(
//         "Player: ({}, {})",
//         player.translation.x, player.translation.y
//     );
// }

// fn spawn_player(mut commands: Commands, sprites: NonSend<Option<Sprites>>) {
//     let sprites = sprites.as_ref().unwrap();
//     commands.spawn((
//         Transform::from_xyz(98., 128., 0.),
//         sprites.player.clone(),
//         Player,
//         Jumps {
//             max: 2,
//             ..default()
//         },
//     ));
// }

// fn reset_jumps(mut query: Query<(&mut Jumps, &Transform)>) {
//     for (mut jumps, transform) in &mut query {
//         if transform.translation.y > 127. {
//             jumps.current = 0;
//         }
//     }
// }

// fn flip_player_sprite(mut player: Single<&mut Sprite, With<Player>>, gamepad: Single<&Gamepad>) {
//     if gamepad.just_pressed(GamepadButton::DPadLeft) {
//         player.horizontal_flipped = true;
//     }

//     if gamepad.just_pressed(GamepadButton::DPadRight) {
//         player.horizontal_flipped = false;
//     }
// }

// fn control_player(
//     player: Single<(&mut Velocity, &mut Jumps), With<Player>>,
//     gamepad: Single<&Gamepad>,
//     noise: ResMut<Noise>,
// ) {
//     let (mut velocity, mut jumps) = player.into_inner();

//     if gamepad.pressed(GamepadButton::DPadLeft) {
//         velocity.x -= 1.;

//         noise.play_sound(
//             Some(1),
//             &EnvelopeSettings::new(1, agb::sound::dmg::SoundDirection::Decrease, 1),
//             4,
//             false,
//             4,
//         );
//     }

//     if gamepad.pressed(GamepadButton::DPadRight) {
//         velocity.x += 1.;

//         noise.play_sound(
//             Some(1),
//             &EnvelopeSettings::new(1, agb::sound::dmg::SoundDirection::Decrease, 1),
//             4,
//             false,
//             4,
//         );
//     }

//     if gamepad.just_pressed(GamepadButton::South) && jumps.current < jumps.max {
//         jumps.current += 1;
//         velocity.y = -5.;

//         noise.play_sound(
//             Some(32),
//             &EnvelopeSettings::new(1, agb::sound::dmg::SoundDirection::Decrease, 4),
//             0,
//             false,
//             0,
//         );
//     }

//     velocity.x = velocity.x.clamp(-2., 2.);
// }

// fn clamp_player_to_screen(mut player: Single<&mut Transform, With<Player>>) {
//     player.translation.x = player.translation.x.clamp(0., 208.);
//     player.translation.y = player.translation.y.clamp(0., 128.);
// }
