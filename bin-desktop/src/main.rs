use std::{
    f32::consts::{FRAC_PI_2, PI, TAU},
    time::Duration,
};

use bevy::{
    color::palettes::css::{
        BLACK, BLUE, FUCHSIA, GREEN, HOT_PINK, LIME, NAVY, ORANGE, ORANGE_RED, PINK, PURPLE,
        REBECCA_PURPLE, RED, TEAL, YELLOW, YELLOW_GREEN,
    },
    dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin},
    ecs::{event, schedule::Stepping},
    log::LogPlugin,
    math::VectorSpace,
    prelude::*,
};
use mel0n::{
    Collider, Mel0nBasePlugin, Mel0nPhysicsSet, Mel0nSetupSet, Root, Velocity,
    fruit::{Collided, FRUIT_DIAMETER, Fruit},
    physics::ImpulseGizmoEvent,
    wall::{Wall, WallLocation},
};
use ops::atan2;

use crate::gamepad_vis::GamepadVisPlugin;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyRoundGizmos {}

fn main() {
    let mut stepping = Stepping::new();
    stepping.add_schedule(FixedUpdate);

    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "info,wgpu_core=warn,wgpu_hal=warn,mygame=debug".into(),
                level: bevy::log::Level::DEBUG,
                ..default()
            }),
            Mel0nBasePlugin,
            MeshPickingPlugin,
            DebugPickingPlugin, // GamepadVisPlugin,
        ))
        .add_systems(
            Update,
            ((
                draw_velocities,
                // draw_velocities_added,
                draw_collision_count,
                draw_impulse_gizmos,
                // angle_draw,
            )
                .chain()
                .after(Mel0nPhysicsSet)),
        )
        .add_systems(Update, stepping_handler)
        .init_gizmo_group::<MyRoundGizmos>()
        .insert_resource(DebugPickingMode::Noisy)
        .insert_resource(Time::<Virtual>::from_max_delta(Duration::from_secs(5)))
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .insert_resource(stepping)
        .insert_resource(ImpulseCache::default())
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .add_systems(Startup, (setup_camera, show_walls.after(Mel0nSetupSet)))
        .run();
}
fn stepping_handler(mut stepping: ResMut<Stepping>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Digit1) || input.pressed(KeyCode::Digit2) {
        // Pressing 1 runs the systems for one frame.
        // Holding 2 runs the systems until the key is released.
        stepping.continue_frame();
    } else if input.just_pressed(KeyCode::Digit3) {
        // Pressing 3 disables stepping which means that the systems run freely.
        stepping.disable();
    } else if input.just_pressed(KeyCode::Digit4) {
        // Pressing 4 enables stepping again.
        stepping.enable();
    }
}
// Thanks https://rparrett.github.io/zola-test/posts/drawing-lines/
fn line_segment(start: Vec2, end: Vec2, thickness: f32, color: Color) -> impl Bundle {
    let length = start.distance(end);
    let diff = start - end;
    let theta = diff.y.atan2(diff.x);
    let midpoint = (start + end) / 2.;

    let transform =
        Transform::from_xyz(midpoint.x, midpoint.y, 0.).with_rotation(Quat::from_rotation_z(theta));

    (
        Sprite {
            color,
            custom_size: Some(Vec2::new(length, thickness)),
            ..default()
        },
        transform,
    )
}

fn draw_velocities(query: Query<(&Velocity, &Transform), With<Fruit>>, mut gizmos: Gizmos) {
    let mirror_y = vec2(1., -1.);
    let cam_offset = vec2(-110.0, 0.);

    for (vel, trans) in query {
        let pos = trans.translation.xy();
        gizmos.arrow_2d(
            pos * mirror_y + cam_offset,
            (pos + (vel.0 * 100.0)) * mirror_y + cam_offset,
            RED,
        );
    }
}

fn draw_velocities_added(query: Query<(&Velocity), With<Fruit>>, mut gizmos: Gizmos) {
    let mirror_y = vec2(1., -1.);
    let cam_offset = vec2(-110.0, 0.);
    let off = |v| v * mirror_y + cam_offset;

    let velocities: Vec<_> = query.iter().collect();

    let mut prev_velocity = Vec2::ZERO;
    let mut offset = prev_velocity;

    for velocity in velocities {
        gizmos.arrow_2d(off(offset), off(offset + (velocity.0 * 20.0)), RED);
        prev_velocity = velocity.0 * 20.0;
        offset += prev_velocity;
    }
}

#[derive(Resource, Default)]
struct ImpulseCache(Option<Vec<ImpulseGizmoEvent>>);

fn draw_impulse_gizmos(
    mut ev_impulse: EventReader<ImpulseGizmoEvent>,
    mut imp_time: ResMut<ImpulseCache>,
    mut gizmos: Gizmos,
) {
    let mirror_y = vec2(1., -1.);
    let cam_offset = vec2(-110.0, 0.);

    let vec = if ev_impulse.is_empty() {
        if let Some(vec) = imp_time.0.clone() {
            vec
        } else {
            return;
        }
    } else {
        let impulses: Vec<ImpulseGizmoEvent> = ev_impulse.read().copied().collect();

        imp_time.0 = Some(impulses.clone());
        impulses
        // for ImpulseGizmoEvent { pos, imp, mass } in impulses {
        //     gizmos.arrow_2d(
        //         pos * mirror_y + cam_offset,
        //         (pos + (imp / mass * 1000.0)) * mirror_y + cam_offset,
        //         GREEN,
        //     );
        // }
    };

    for ImpulseGizmoEvent { pos, imp, mass } in vec {
        gizmos.arrow_2d(
            pos * mirror_y + cam_offset,
            (pos + (imp / mass * 100.0)) * mirror_y + cam_offset,
            GREEN,
        );
    }
}

fn angle_draw(query: Query<(&Transform), With<Fruit>>, mut gizmos: Gizmos) {
    let mirror_y = vec2(1., -1.);
    let cam_offset = vec2(-110.0, 0.);
    let off = |v| v * mirror_y + cam_offset;

    let mut fruits = query.iter();
    let Some(Transform { translation: a, .. }) = fruits.next() else {
        return;
    };
    let Some(Transform { translation: b, .. }) = fruits.next() else {
        return;
    };

    // log::info!("ang {}", a.xy().angle_to(b.xy()).to_degrees());
    let normal_dir = a.xy().angle_to(b.xy());
    let normal = Vec2::from_angle(normal_dir - FRAC_PI_2).normalize();

    let better_angle = atan2(b.y - a.y, b.x - a.x);
    gizmos.arrow_2d(off(a.xy()), off(b.xy()), PURPLE);
    gizmos.arrow_2d(off(vec2(0., 0.)), off(normal * 100.0), PINK);
    gizmos.arrow_2d(
        off(vec2(0., 0.)),
        off(Vec2::from_angle(better_angle) * 100.0),
        REBECCA_PURPLE,
    );

    // for (vel, trans) in query {
    //     let pos = trans.translation.xy();
    //     gizmos.arrow_2d(
    //         pos * mirror_y + cam_offset,
    //         (pos + (vel.0 * 10.0)) * mirror_y + cam_offset,
    //         RED,
    //     );
    // }
}
fn draw_collision_count(query: Query<(&Collided, &Transform), With<Fruit>>, mut gizmos: Gizmos) {
    let mirror_y = vec2(1., -1.);
    let cam_offset = vec2(-110.0, 0.);

    let rainbow = [RED, ORANGE_RED, YELLOW, GREEN, BLUE, PURPLE, HOT_PINK];

    for (cold, trans) in query {
        let pos = trans.translation.xy();
        gizmos.circle_2d(
            pos * mirror_y + cam_offset,
            FRUIT_DIAMETER / 2.0,
            rainbow[(cold.0 % 7) as usize],
        );
    }
}

fn kill_busy_fruits(mut commands: Commands, query: Query<(Entity, &Collided), With<Fruit>>) {
    for (ent, collided) in query {
        if collided.0 > 10 {
            commands.entity(ent).despawn();
        }
    }
}

fn setup_camera(mut commands: Commands) {
    info!("Starting!");

    commands.spawn((
        Camera2d,
        // Projection::custom(Projection::Orthographic(OrthographicProjection {
        //     area: Rect::new(-1.0, 1.0, 1.0, -1.0),
        //     scaling_mode: ScalingMode::FixedVertical {}, ..OrthographicProjection::default_2d(),
        // })),
    ));
}

fn show_walls(
    mut commands: Commands,
    query: Query<(&Wall, &Transform, &Collider)>,
    root: Single<Entity, With<Root>>,
) {
    let root = root.entity();

    for (_, transform, collider) in query {
        let horizontal = collider.half_size.y < collider.half_size.x;

        let offset = if horizontal {
            vec2(120.0 - 8.0, 0. - 8.0)
        } else {
            vec2(0.0 - 8.0, 74.0 - 8.0)
        };
        let entity = commands
            .spawn((
                Sprite::from_color(Color::linear_rgb(0.1, 0.1, 0.1), Vec2::ONE),
                transform
                    .with_scale((collider.half_size * 2.).extend(1.0))
                    .with_translation(transform.translation + offset.extend(0.0)),
            ))
            .id();
        commands.entity(root).add_child(entity);
    }
}
//
// fn add_debugs(mut commands: Commands, query: Query<(&Velocity, With<Fruit>)>) {
//     for (_, transform, collider) in query {
//         commands.spawn((
//             Sprite::from_color(Color::linear_rgb(0.1, 0.1, 0.1), Vec2::ONE),
//             transform.clone().with_scale(vec3(-1., 1., 1.)).with_scale((collider.half_size * 2.).extend(-1.0))
//         ));
//     }
// }
//
// fn debug_velocities(mut commands: Commands, query: Query<(&Velocity, With<Fruit>)>) {
//     for (_, transform, collider) in query {
//         commands.spawn((
//             Sprite::from_color(Color::linear_rgb(0.1, 0.1, 0.1), Vec2::ONE),
//             transform.clone().with_scale(vec3(-1., 1., 1.)).with_scale((collider.half_size * 2.).extend(-1.0))
//         ));
//     }
// }

mod gamepad_vis {
    use std::f32::consts::PI;

    use bevy::{
        input::gamepad::{
            GamepadAxisChangedEvent, GamepadButtonChangedEvent, GamepadConnectionEvent,
        },
        prelude::*,
        sprite::Anchor,
    };

    const BUTTON_RADIUS: f32 = 25.;
    const BUTTON_CLUSTER_RADIUS: f32 = 50.;
    const START_SIZE: Vec2 = Vec2::new(30., 15.);
    const TRIGGER_SIZE: Vec2 = Vec2::new(70., 20.);
    const STICK_BOUNDS_SIZE: f32 = 100.;

    const BUTTONS_X: f32 = 150.;
    const BUTTONS_Y: f32 = 80.;
    const STICKS_X: f32 = 150.;
    const STICKS_Y: f32 = -135.;

    const NORMAL_BUTTON_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
    const ACTIVE_BUTTON_COLOR: Color = Color::srgb(0.5, 0., 0.5);
    const LIVE_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
    const DEAD_COLOR: Color = Color::srgb(0.13, 0.13, 0.13);

    #[derive(Component, Deref)]
    struct ReactTo(GamepadButton);
    #[derive(Component)]
    struct MoveWithAxes {
        x_axis: GamepadAxis,
        y_axis: GamepadAxis,
        scale: f32,
    }
    #[derive(Component)]
    struct TextWithAxes {
        x_axis: GamepadAxis,
        y_axis: GamepadAxis,
    }
    #[derive(Component, Deref)]
    struct TextWithButtonValue(GamepadButton);

    #[derive(Component)]
    struct ConnectedGamepadsText;

    #[derive(Resource)]
    struct ButtonMaterials {
        normal: MeshMaterial2d<ColorMaterial>,
        active: MeshMaterial2d<ColorMaterial>,
    }
    impl FromWorld for ButtonMaterials {
        fn from_world(world: &mut World) -> Self {
            Self {
                normal: world.add_asset(NORMAL_BUTTON_COLOR).into(),
                active: world.add_asset(ACTIVE_BUTTON_COLOR).into(),
            }
        }
    }
    #[derive(Resource)]
    struct ButtonMeshes {
        circle: Mesh2d,
        triangle: Mesh2d,
        start_pause: Mesh2d,
        trigger: Mesh2d,
    }
    impl FromWorld for ButtonMeshes {
        fn from_world(world: &mut World) -> Self {
            Self {
                circle: world.add_asset(Circle::new(BUTTON_RADIUS)).into(),
                triangle: world
                    .add_asset(RegularPolygon::new(BUTTON_RADIUS, 3))
                    .into(),
                start_pause: world.add_asset(Rectangle::from_size(START_SIZE)).into(),
                trigger: world.add_asset(Rectangle::from_size(TRIGGER_SIZE)).into(),
            }
        }
    }

    #[derive(Bundle)]
    struct GamepadButtonBundle {
        mesh: Mesh2d,
        material: MeshMaterial2d<ColorMaterial>,
        transform: Transform,
        react_to: ReactTo,
    }

    impl GamepadButtonBundle {
        pub fn new(
            button_type: GamepadButton,
            mesh: Mesh2d,
            material: MeshMaterial2d<ColorMaterial>,
            x: f32,
            y: f32,
        ) -> Self {
            Self {
                mesh,
                material,
                transform: Transform::from_xyz(x, y, 0.),
                react_to: ReactTo(button_type),
            }
        }

        pub fn with_rotation(mut self, angle: f32) -> Self {
            self.transform.rotation = Quat::from_rotation_z(angle);
            self
        }
    }

    pub struct GamepadVisPlugin;

    impl Plugin for GamepadVisPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<ButtonMaterials>()
                .init_resource::<ButtonMeshes>()
                .add_systems(
                    Startup,
                    (setup, setup_sticks, setup_triggers, setup_connected),
                )
                .add_systems(
                    Update,
                    (
                        update_buttons,
                        update_button_values,
                        update_axes,
                        update_connected,
                    ),
                );
        }
    }

    fn setup(mut commands: Commands, meshes: Res<ButtonMeshes>, materials: Res<ButtonMaterials>) {
        // Buttons

        commands.spawn((
            Transform::from_xyz(BUTTONS_X, BUTTONS_Y, 0.),
            Visibility::default(),
            children![
                GamepadButtonBundle::new(
                    GamepadButton::North,
                    meshes.circle.clone(),
                    materials.normal.clone(),
                    0.,
                    BUTTON_CLUSTER_RADIUS,
                ),
                GamepadButtonBundle::new(
                    GamepadButton::South,
                    meshes.circle.clone(),
                    materials.normal.clone(),
                    0.,
                    -BUTTON_CLUSTER_RADIUS,
                ),
                GamepadButtonBundle::new(
                    GamepadButton::West,
                    meshes.circle.clone(),
                    materials.normal.clone(),
                    -BUTTON_CLUSTER_RADIUS,
                    0.,
                ),
                GamepadButtonBundle::new(
                    GamepadButton::East,
                    meshes.circle.clone(),
                    materials.normal.clone(),
                    BUTTON_CLUSTER_RADIUS,
                    0.,
                ),
            ],
        ));

        // Start and Pause

        commands.spawn(GamepadButtonBundle::new(
            GamepadButton::Select,
            meshes.start_pause.clone(),
            materials.normal.clone(),
            -30.,
            BUTTONS_Y,
        ));

        commands.spawn(GamepadButtonBundle::new(
            GamepadButton::Start,
            meshes.start_pause.clone(),
            materials.normal.clone(),
            30.,
            BUTTONS_Y,
        ));

        // D-Pad

        commands.spawn((
            Transform::from_xyz(-BUTTONS_X, BUTTONS_Y, 0.),
            Visibility::default(),
            children![
                GamepadButtonBundle::new(
                    GamepadButton::DPadUp,
                    meshes.triangle.clone(),
                    materials.normal.clone(),
                    0.,
                    BUTTON_CLUSTER_RADIUS,
                ),
                GamepadButtonBundle::new(
                    GamepadButton::DPadDown,
                    meshes.triangle.clone(),
                    materials.normal.clone(),
                    0.,
                    -BUTTON_CLUSTER_RADIUS,
                )
                .with_rotation(PI),
                GamepadButtonBundle::new(
                    GamepadButton::DPadLeft,
                    meshes.triangle.clone(),
                    materials.normal.clone(),
                    -BUTTON_CLUSTER_RADIUS,
                    0.,
                )
                .with_rotation(PI / 2.),
                GamepadButtonBundle::new(
                    GamepadButton::DPadRight,
                    meshes.triangle.clone(),
                    materials.normal.clone(),
                    BUTTON_CLUSTER_RADIUS,
                    0.,
                )
                .with_rotation(-PI / 2.),
            ],
        ));

        // Triggers

        commands.spawn(GamepadButtonBundle::new(
            GamepadButton::LeftTrigger,
            meshes.trigger.clone(),
            materials.normal.clone(),
            -BUTTONS_X,
            BUTTONS_Y + 115.,
        ));

        commands.spawn(GamepadButtonBundle::new(
            GamepadButton::RightTrigger,
            meshes.trigger.clone(),
            materials.normal.clone(),
            BUTTONS_X,
            BUTTONS_Y + 115.,
        ));
    }

    fn setup_sticks(
        mut commands: Commands,
        meshes: Res<ButtonMeshes>,
        materials: Res<ButtonMaterials>,
    ) {
        // NOTE: This stops making sense because in entities because there isn't a "global" default,
        // instead each gamepad has its own default setting
        let gamepad_settings = GamepadSettings::default();
        let dead_upper =
            STICK_BOUNDS_SIZE * gamepad_settings.default_axis_settings.deadzone_upperbound();
        let dead_lower =
            STICK_BOUNDS_SIZE * gamepad_settings.default_axis_settings.deadzone_lowerbound();
        let dead_size = dead_lower.abs() + dead_upper.abs();
        let dead_mid = (dead_lower + dead_upper) / 2.0;

        let live_upper =
            STICK_BOUNDS_SIZE * gamepad_settings.default_axis_settings.livezone_upperbound();
        let live_lower =
            STICK_BOUNDS_SIZE * gamepad_settings.default_axis_settings.livezone_lowerbound();
        let live_size = live_lower.abs() + live_upper.abs();
        let live_mid = (live_lower + live_upper) / 2.0;

        let mut spawn_stick = |x_pos, y_pos, x_axis, y_axis, button| {
            let style = TextFont {
                font_size: 13.,
                ..default()
            };
            commands.spawn((
                Transform::from_xyz(x_pos, y_pos, 0.),
                Visibility::default(),
                children![
                    Sprite::from_color(DEAD_COLOR, Vec2::splat(STICK_BOUNDS_SIZE * 2.),),
                    (
                        Sprite::from_color(LIVE_COLOR, Vec2::splat(live_size)),
                        Transform::from_xyz(live_mid, live_mid, 2.),
                    ),
                    (
                        Sprite::from_color(DEAD_COLOR, Vec2::splat(dead_size)),
                        Transform::from_xyz(dead_mid, dead_mid, 3.),
                    ),
                    (
                        Text2d::default(),
                        Transform::from_xyz(0., STICK_BOUNDS_SIZE + 2., 4.),
                        Anchor::BottomCenter,
                        TextWithAxes { x_axis, y_axis },
                        children![
                            (TextSpan(format!("{:.3}", 0.)), style.clone()),
                            (TextSpan::new(", "), style.clone()),
                            (TextSpan(format!("{:.3}", 0.)), style),
                        ]
                    ),
                    (
                        meshes.circle.clone(),
                        materials.normal.clone(),
                        Transform::from_xyz(0., 0., 5.).with_scale(Vec2::splat(0.15).extend(1.)),
                        MoveWithAxes {
                            x_axis,
                            y_axis,
                            scale: STICK_BOUNDS_SIZE,
                        },
                        ReactTo(button),
                    ),
                ],
            ));
        };

        spawn_stick(
            -STICKS_X,
            STICKS_Y,
            GamepadAxis::LeftStickX,
            GamepadAxis::LeftStickY,
            GamepadButton::LeftThumb,
        );
        spawn_stick(
            STICKS_X,
            STICKS_Y,
            GamepadAxis::RightStickX,
            GamepadAxis::RightStickY,
            GamepadButton::RightThumb,
        );
    }

    fn setup_triggers(
        mut commands: Commands,
        meshes: Res<ButtonMeshes>,
        materials: Res<ButtonMaterials>,
    ) {
        let mut spawn_trigger = |x, y, button_type| {
            commands.spawn((
                GamepadButtonBundle::new(
                    button_type,
                    meshes.trigger.clone(),
                    materials.normal.clone(),
                    x,
                    y,
                ),
                children![(
                    Transform::from_xyz(0., 0., 1.),
                    Text(format!("{:.3}", 0.)),
                    TextFont {
                        font_size: 13.,
                        ..default()
                    },
                    TextWithButtonValue(button_type),
                )],
            ));
        };

        spawn_trigger(-BUTTONS_X, BUTTONS_Y + 145., GamepadButton::LeftTrigger2);
        spawn_trigger(BUTTONS_X, BUTTONS_Y + 145., GamepadButton::RightTrigger2);
    }

    fn setup_connected(mut commands: Commands) {
        // This is UI text, unlike other text in this example which is 2d.
        commands.spawn((
            Text::new("Connected Gamepads:\n"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.),
                left: Val::Px(12.),
                ..default()
            },
            ConnectedGamepadsText,
            children![TextSpan::new("None")],
        ));
    }

    fn update_buttons(
        gamepads: Query<&Gamepad>,
        materials: Res<ButtonMaterials>,
        mut query: Query<(&mut MeshMaterial2d<ColorMaterial>, &ReactTo)>,
    ) {
        for gamepad in &gamepads {
            for (mut handle, react_to) in query.iter_mut() {
                if gamepad.just_pressed(**react_to) {
                    *handle = materials.active.clone();
                }
                if gamepad.just_released(**react_to) {
                    *handle = materials.normal.clone();
                }
            }
        }
    }
    fn update_button_values(
        mut events: EventReader<GamepadButtonChangedEvent>,
        mut query: Query<(&mut Text2d, &TextWithButtonValue)>,
    ) {
        for button_event in events.read() {
            for (mut text, text_with_button_value) in query.iter_mut() {
                if button_event.button == **text_with_button_value {
                    **text = format!("{:.3}", button_event.value);
                }
            }
        }
    }

    fn update_axes(
        mut axis_events: EventReader<GamepadAxisChangedEvent>,
        mut query: Query<(&mut Transform, &MoveWithAxes)>,
        text_query: Query<(Entity, &TextWithAxes)>,
        mut writer: Text2dWriter,
    ) {
        for axis_event in axis_events.read() {
            let axis_type = axis_event.axis;
            let value = axis_event.value;
            for (mut transform, move_with) in query.iter_mut() {
                if axis_type == move_with.x_axis {
                    transform.translation.x = value * move_with.scale;
                }
                if axis_type == move_with.y_axis {
                    transform.translation.y = value * move_with.scale;
                }
            }
            for (text, text_with_axes) in text_query.iter() {
                if axis_type == text_with_axes.x_axis {
                    *writer.text(text, 1) = format!("{value:.3}");
                }
                if axis_type == text_with_axes.y_axis {
                    *writer.text(text, 3) = format!("{value:.3}");
                }
            }
        }
    }

    fn update_connected(
        mut connected: EventReader<GamepadConnectionEvent>,
        gamepads: Query<(Entity, &Name), With<Gamepad>>,
        text: Single<Entity, With<ConnectedGamepadsText>>,
        mut writer: TextUiWriter,
    ) {
        if connected.is_empty() {
            return;
        }
        connected.clear();

        let formatted = gamepads
            .iter()
            .map(|(entity, name)| format!("{} - {}", entity, name))
            .collect::<Vec<_>>()
            .join("\n");

        *writer.text(*text, 1) = if !formatted.is_empty() {
            formatted
        } else {
            "None".to_string()
        }
    }
}
