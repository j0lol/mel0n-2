use bevy::prelude::*;

use crate::{
    Collider, Velocity,
    fruit::{Diameter, Fruit},
    physics::Physics,
};

const WALL_THICKNESS: f32 = 1.;
const LEFT_WALL: f32 = 62.;
const RIGHT_WALL: f32 = 179. - WALL_THICKNESS;
// y coordinates
const BOTTOM_WALL: f32 = 148. - WALL_THICKNESS;
const TOP_WALL: f32 = 0.;

// This is a collection of the components that define a "Wall" in our game
#[derive(Component, Debug)]
#[require(Transform)]
pub struct Wall;

/// Which side of the arena is this wall located on?
pub enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    /// Location of the *center* of the wall, used in `transform.translation()`
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    /// (x, y) dimensions of the wall, used in `transform.scale()`
    fn size(&self) -> Vec2 {
        let arena_height = BOTTOM_WALL - TOP_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl Wall {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic.
    // Notice the use of Sprite and Transform alongside Wall, overwriting the default values defined for the required components
    fn new(location: WallLocation) -> (Wall, Transform, Collider, Physics) {
        (
            Wall,
            Transform {
                // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                // This is used to determine the order of our sprites
                translation: location.position().extend(0.0),
                ..default()
            },
            Collider {
                half_size: location.size() / 2.,
            },
            Physics,
        )
    }
}

pub fn add_walls(mut commands: Commands) {
    commands.spawn(Wall::new(WallLocation::Left));
    commands.spawn(Wall::new(WallLocation::Right));
    commands.spawn(Wall::new(WallLocation::Bottom));
    commands.spawn(Wall::new(WallLocation::Top));
}

pub fn constrain_objects(query: Query<(&mut Transform, &mut Velocity, &Diameter), With<Fruit>>) {
    // log::info!("bwuh");

    for (mut ts, mut vl, dm) in query {
        // log::info!("guh {:?}", ts.0.translation);

        if ts.translation.x != ts.translation.x.clamp(LEFT_WALL, RIGHT_WALL - dm.0) {
            vl.0.x *= -0.2;
        }
        if ts.translation.y != ts.translation.y.clamp(-9999.0, BOTTOM_WALL - dm.0) {
            vl.0.y *= -0.2;
        }

        ts.translation.x = ts.translation.x.clamp(LEFT_WALL, RIGHT_WALL - dm.0);
        ts.translation.y = ts.translation.y.clamp(-9999.0, BOTTOM_WALL - dm.0);
    }
}
