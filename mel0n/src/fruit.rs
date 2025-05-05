use bevy::prelude::*;

#[cfg(feature = "gba")]
use crate::Sprites;
use crate::{
    Gravity, Root, Velocity,
    physics::{ActingForces, Physics},
};

#[derive(Component, Clone, Copy, Default, Debug)]
pub struct Diameter(pub f32);

#[derive(Component, Clone, Copy, Default, Debug)]
pub struct Collided(pub u32);

#[derive(Component, Default, Debug)]
pub struct Fruit;

#[derive(Bundle, Default, Debug)]
pub struct FruitBundle {
    marker: Fruit,
    transform: Transform,
    velocity: Velocity,
    acting_forces: ActingForces,
    grav_marker: Gravity,
    diameter: Diameter,
    physics: Physics,
}

// static FRUIT_POS: [Vec2; 2] = [vec2(90., 999.), vec2(90., 30.)];
static FRUIT_POS: [Vec2; 0] = [];

pub static FRUIT_DIAMETER: f32 = 16.;
#[cfg(feature = "gba")]
pub fn add_fruit(
    mut commands: Commands,
    sprites: NonSend<Option<Sprites>>,
    root: Single<Entity, With<Root>>,
) {
    let sprites = sprites.as_ref().unwrap();

    for fruit in FRUIT_POS {
        let entity = commands
            .spawn((
                FruitBundle {
                    transform: Transform::from_translation(fruit.extend(1.0))
                        .with_scale(Vec2::splat(FRUIT_DIAMETER).extend(1.)),
                    diameter: Diameter(FRUIT_DIAMETER),
                    ..default()
                },
                sprites.player.clone(),
                Collided(0),
            ))
            .id();

        commands.entity(*root).add_child(entity);
    }
}
#[cfg(feature = "desktop")]
pub fn add_fruit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    root: Single<Entity, With<Root>>,
) {
    for fruit in FRUIT_POS {
        let entity = commands
            .spawn((
                FruitBundle {
                    diameter: Diameter(FRUIT_DIAMETER),
                    transform: Transform::from_translation(fruit.extend(1.0))
                        .with_scale(Vec2::splat(FRUIT_DIAMETER).extend(1.)),
                    ..default()
                },
                Mesh2d(meshes.add(Circle::default())),
                MeshMaterial2d(materials.add(Color::linear_rgb(1.0, 0.0, 1.0))),
                Collided(0),
            ))
            .id();

        commands.entity(*root).add_child(entity);
    }
}

#[cfg(feature = "gba")]
pub fn place_fruit(
    gamepad: Single<&Gamepad>,
    mut commands: Commands,
    sprites: NonSend<Option<Sprites>>,
    root: Single<Entity, With<Root>>,
) {
    if gamepad.just_pressed(GamepadButton::East) {
        let sprites = sprites.as_ref().unwrap();

        let fruit = vec2(100., 0.);

        let entity = commands
            .spawn((
                FruitBundle {
                    transform: Transform::from_xyz(fruit.x, fruit.y, 0.0),
                    diameter: Diameter(FRUIT_DIAMETER),
                    ..default()
                },
                sprites.player.clone(),
                Collided(0),
            ))
            .id();

        commands.entity(*root).add_child(entity);
        return;
    }
}

#[cfg(feature = "desktop")]
pub fn place_fruit(
    // gamepad: Single<&Gamepad>,
    gamepad: Option<Single<(&Gamepad)>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    root: Single<Entity, With<Root>>,
) {
    // let (root, _) = *root;
    // info!("{n:?}");

    use crate::MOON_PHYSICS;
    if gamepad.is_some_and(|g| g.just_pressed(GamepadButton::East))
        || keys.just_pressed(KeyCode::Space)
    {
        let fruit = vec2(100., 0.);

        let entity = commands
            .spawn((
                FruitBundle {
                    diameter: Diameter(FRUIT_DIAMETER),
                    transform: Transform::from_translation(fruit.extend(1.0))
                        .with_scale(Vec2::splat(FRUIT_DIAMETER).extend(1.)),
                    velocity: Velocity(vec2(
                        if MOON_PHYSICS { 0.7 } else { 0.0 },
                        if MOON_PHYSICS { 1.0 } else { 0.0 },
                    )),
                    ..default()
                },
                Mesh2d(meshes.add(Circle::default())),
                MeshMaterial2d(materials.add(Color::linear_rgb(1.0, 0.0, 1.0))),
                Collided(0),
            ))
            .observe(on_click_delete_fruit)
            .id();
        commands.entity(*root).add_child(entity);
        return;
    }

    // if gamepad.just_pressed(GamepadButton::East) {
    //     add_fruit(commands, meshes, materials);
    // }
}

#[cfg(feature = "desktop")]
pub fn on_drag_move_fruit(
    drag: Trigger<Pointer<Drag>>,
    mut transforms: Query<&mut Transform, With<Fruit>>,
) {
    if let Ok(mut transform) = transforms.get_mut(drag.target()) {
        transform.translation.x += (drag.delta.x);
        transform.translation.y += (drag.delta.y);
    }
}

#[cfg(feature = "desktop")]
pub fn on_click_delete_fruit(
    click: Trigger<Pointer<Click>>,
    mut entities: Query<Entity>,
    mut commands: Commands,
) -> Result<()> {
    let entity = entities.get(click.target())?;

    commands.entity(entity).despawn();

    Ok(())
}
