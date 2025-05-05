//! An example game written in the Bevy game engine and using the [`agb`] crate to allow running it
//! on the Game Boy Advance.

//! We declare our crate as `no_std`, as the Game Boy Advance doesn't have a port of the standard
//! library.
#![no_std]

pub mod fruit;
#[cfg(feature = "gba")]
pub mod gba;
pub mod physics;
pub mod wall;

use bevy::prelude::*;
#[cfg(feature = "gba")]
use bevy_mod_gba::Sprite;
use fruit::add_fruit;
#[cfg(feature = "gba")]
use gba::Mel0nGbaSetupSet;
use physics::{apply_collisions, apply_friction, apply_gravity, integrate_position};
use wall::add_walls;

use crate::{fruit::place_fruit, wall::constrain_objects};

#[derive(Component)]
#[require(Gravity, Jumps, Velocity, Transform)]
pub struct Player;

#[derive(Component, Default, Debug)]
#[require(Velocity)]
pub struct Gravity;

#[derive(Component, Default, Debug)]
#[require(Transform)]
pub struct Velocity(pub Vec2);

#[derive(Component, Default)]
pub struct Jumps {
    current: u8,
    max: u8,
}

#[derive(Component)]
pub struct Collider {
    pub half_size: Vec2,
}

#[cfg(feature = "gba")]
pub struct Sprites {
    player: Sprite,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mel0nSetupSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mel0nPhysicsSet;

pub struct Mel0nBasePlugin;

impl Plugin for Mel0nBasePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "gba")]
        app.configure_sets(Startup, Mel0nSetupSet.after(Mel0nGbaSetupSet));

        app.add_systems(
            Startup,
            (make_root, (add_fruit, add_walls).in_set(Mel0nSetupSet)).chain(),
        );

        // .add_systems(Update, log_player_position)
        app.add_systems(
            Update,
            (
                // control_player,
                // flip_player_sprite,
                //
                (
                    apply_gravity,
                    apply_friction,
                    integrate_position,
                    apply_collisions,
                    constrain_objects,
                )
                    .chain()
                    .in_set(Mel0nPhysicsSet),
                place_fruit,
                // clamp_player_to_screen,
                // reset_jumps,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
pub struct Root;

fn make_root(mut commands: Commands) {
    commands.spawn((
        Name::new("Root"),
        Root,
        Transform::from_xyz(
            {
                #[cfg(feature = "desktop")]
                {
                    -110.0
                }
                #[cfg(not(feature = "desktop"))]
                {
                    0.0
                }
            },
            0.,
            1.0,
        )
        .with_scale(Vec3::new(
            1.0,
            {
                #[cfg(feature = "desktop")]
                {
                    -1.0
                }
                #[cfg(not(feature = "desktop"))]
                {
                    1.0
                }
            },
            1.0,
        )),
    ));
}
