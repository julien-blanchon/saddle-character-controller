use std::{fmt::Write as _, time::Duration};

use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_enhanced_input::prelude::*;
use saddle_character_controller::{
    AccumulatedInput, AscendAction, CharacterController, CharacterControllerPlugin,
    CharacterControllerState, CharacterControllerSystems, CharacterLook, CharacterMantle,
    CharacterMotionStats, CharacterPush, CharacterSwimming, CharacterWallKick, CrouchAction,
    JumpAction, LookAction, MoveAction, MovementSurface, SprintAction, SupportVelocityPolicy,
    TraverseAction, WaterVolume,
};

#[derive(Component)]
struct LabController;

#[derive(Component)]
struct LabCamera {
    distance: f32,
    height: f32,
}

#[derive(Component)]
struct LabOverlay;

#[derive(Component)]
struct MovingPlatform {
    origin: Vec3,
    translation_axis: Vec3,
    translation_amplitude: f32,
    translation_speed: f32,
    rotation_speed: f32,
    phase: f32,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LabFixedSystems {
    AnimatePlatforms,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.add_plugins((
        DefaultPlugins,
        PhysicsPlugins::default(),
        CharacterControllerPlugin::always_on(FixedUpdate),
    ));
    app.configure_sets(
        FixedUpdate,
        LabFixedSystems::AnimatePlatforms.before(CharacterControllerSystems::Grounding),
    );
    app.add_systems(Startup, setup);
    app.add_systems(
        FixedUpdate,
        animate_platforms.in_set(LabFixedSystems::AnimatePlatforms),
    );
    app.add_systems(
        Update,
        (
            sync_overlay,
            capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
            release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
        ),
    );
    app.add_systems(PostUpdate, follow_camera);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Lab Sun"),
        DirectionalLight {
            illuminance: 22_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(12.0, 18.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Name::new("Lab Ground"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(80.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.18, 0.17),
            perceptual_roughness: 1.0,
            ..default()
        })),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Mantle Block",
        Vec3::new(0.0, 0.8, -2.0),
        Vec3::new(2.2, 1.6, 2.2),
        Color::srgb(0.36, 0.57, 0.44),
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Walkable Ramp",
        Vec3::new(-8.0, 1.1, -2.5),
        Vec3::new(8.0, 0.8, 8.0),
        -0.42,
        Color::srgb(0.25, 0.50, 0.42),
        None,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Surf Ramp",
        Vec3::new(12.0, 2.0, 2.5),
        Vec3::new(12.0, 0.5, 10.0),
        -1.0,
        Color::srgb(0.72, 0.35, 0.29),
        Some(MovementSurface {
            slide_only: true,
            traction_multiplier: 0.25,
            ..default()
        }),
    );
    spawn_stairs(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-16.0, 0.0, -10.0),
        6,
        1.1,
        0.26,
        2.4,
    );
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Platform A",
        Vec3::new(-6.0, 1.2, 5.0),
        Vec3::new(3.5, 0.35, 3.5),
        Color::srgb(0.81, 0.58, 0.22),
        Some(MovingPlatform {
            origin: Vec3::new(-6.0, 1.2, 5.0),
            translation_axis: Vec3::X,
            translation_amplitude: 4.0,
            translation_speed: 0.7,
            rotation_speed: 0.25,
            phase: 0.0,
        }),
        Some(MovementSurface {
            inherit_velocity_policy: Some(SupportVelocityPolicy::Full),
            ..default()
        }),
    );
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Conveyor",
        Vec3::new(10.0, 0.35, -8.0),
        Vec3::new(8.0, 0.3, 3.0),
        Color::srgb(0.41, 0.38, 0.77),
        None,
        Some(MovementSurface {
            conveyor_velocity: Vec3::new(2.0, 0.0, 0.0),
            inherit_velocity_policy: Some(SupportVelocityPolicy::Horizontal),
            ..default()
        }),
    );
    spawn_water_volume(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Pool",
        Vec3::new(16.0, 1.6, 10.0),
        Vec3::new(10.0, 3.0, 8.0),
    );
    spawn_dynamic_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Push Crate",
        Vec3::new(2.5, 1.0, 3.0),
        Vec3::new(1.2, 1.2, 1.2),
        Color::srgb(0.56, 0.40, 0.28),
    );

    spawn_controller(&mut commands, &mut meshes, &mut materials);

    commands.spawn((
        Name::new("Lab Camera"),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::TAU / 5.6,
            ..default()
        }),
        LabCamera {
            distance: 6.8,
            height: 2.8,
        },
        Transform::from_xyz(0.0, 5.5, 18.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Character Controller Overlay"),
        LabOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(14.0)),
            border_radius: BorderRadius::all(Val::Px(16.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.78)),
        Text::default(),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn spawn_controller(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    let player = commands
        .spawn((
            Name::new("Lab Controller"),
            LabController,
            CharacterController {
                support_velocity_policy: SupportVelocityPolicy::Full,
                support_detach_grace: Duration::from_millis(180),
                coyote_time: Duration::from_millis(130),
                jump_input_buffer: Duration::from_millis(170),
                ..default()
            },
            CharacterLook {
                sensitivity: Vec2::splat(0.0023),
                ..default()
            },
            CharacterSwimming::default(),
            CharacterMantle::default(),
            CharacterWallKick::default(),
            CharacterPush::default(),
            Transform::from_xyz(0.0, 2.5, 14.0),
            actions!(CharacterController[
                (
                    Action::<MoveAction>::new(),
                    DeadZone::default(),
                    Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                ),
                (
                    Action::<LookAction>::new(),
                    Bindings::spawn((
                        Spawn((Binding::mouse_motion(), Scale::splat(0.0025))),
                        Axial::right_stick().with((Scale::splat(0.06), DeadZone::default())),
                    )),
                ),
                (Action::<JumpAction>::new(), bindings![KeyCode::Space, GamepadButton::South]),
                (
                    Action::<SprintAction>::new(),
                    bindings![KeyCode::ShiftLeft, GamepadButton::LeftTrigger2],
                ),
                (
                    Action::<CrouchAction>::new(),
                    bindings![KeyCode::ControlLeft, KeyCode::KeyC, GamepadButton::East],
                ),
                (
                    Action::<AscendAction>::new(),
                    bindings![KeyCode::KeyQ, GamepadButton::LeftTrigger],
                ),
                (
                    Action::<TraverseAction>::new(),
                    bindings![KeyCode::KeyE, GamepadButton::RightTrigger],
                ),
            ]),
        ))
        .id();

    commands.entity(player).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.38, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.92, 0.44, 0.24),
                perceptual_roughness: 0.82,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.9, 0.0),
        ));
    });

    player
}

fn follow_camera(
    target: Single<(&Transform, &CharacterControllerState), With<LabController>>,
    mut camera_query: Query<(&LabCamera, &mut Transform), Without<LabController>>,
) {
    let (target_transform, state) = *target;
    let Ok((camera, mut transform)) = camera_query.single_mut() else {
        return;
    };
    let focus = target_transform.translation + Vec3::Y * (camera.height * 0.6);
    let backward = state.orientation * Vec3::Z;
    *transform = Transform::from_translation(
        focus + backward * camera.distance + Vec3::Y * (camera.height * 0.5),
    )
    .looking_at(focus, Vec3::Y);
}

fn sync_overlay(
    controller: Single<
        (
            &Transform,
            &CharacterControllerState,
            &CharacterMotionStats,
            &AccumulatedInput,
        ),
        With<LabController>,
    >,
    mut overlay: Single<&mut Text, With<LabOverlay>>,
    names: Query<&Name>,
) {
    let (transform, state, stats, input) = *controller;
    let text = &mut *overlay;

    let ground_name = state
        .ground
        .and_then(|ground| names.get(ground.entity).ok())
        .map_or_else(|| "none".to_owned(), |name| name.as_str().to_owned());
    let support_name = state
        .ground
        .map(|ground| ground.entity)
        .and_then(|entity| names.get(entity).ok())
        .map_or_else(|| "none".to_owned(), |name| name.as_str().to_owned());
    let jump_buffer = input
        .jump_pressed_for
        .map_or_else(|| "none".to_owned(), |age| format!("{age:.2}s"));
    let traverse_buffer = input
        .traverse_pressed_for
        .map_or_else(|| "none".to_owned(), |age| format!("{age:.2}s"));

    text.0.clear();
    let _ = writeln!(text.0, "Character Controller Lab");
    let _ = writeln!(text.0, "mode: {:?}", state.movement_mode);
    let _ = writeln!(text.0, "ground: {ground_name}");
    let _ = writeln!(
        text.0,
        "support: {support_name} lin={:.2} ang={:.2}",
        state.support_velocity.length(),
        state.support_angular_velocity.length()
    );
    let _ = writeln!(
        text.0,
        "water: {:?} speed={:.2} accel={:.2} gravity={:.2}",
        state.water_level,
        state.water_speed_multiplier,
        state.water_acceleration_multiplier,
        state.water_gravity_multiplier
    );
    let _ = writeln!(
        text.0,
        "pos: ({:.1}, {:.1}, {:.1}) speed={:.2} horiz={:.2}",
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
        stats.current_speed,
        stats.horizontal_speed
    );
    let _ = writeln!(text.0, "casts: {}", stats.shape_casts_last_tick);
    let _ = write!(
        text.0,
        "buffers: jump={jump_buffer} traverse={traverse_buffer}"
    );
}

fn capture_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::None;
    cursor.visible = true;
}

fn animate_platforms(
    time: Res<Time<Fixed>>,
    mut query: Query<(
        &MovingPlatform,
        &mut Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
    )>,
) {
    let t = time.elapsed_secs();
    for (platform, mut transform, mut linear_velocity, mut angular_velocity) in &mut query {
        let axis = platform.translation_axis.normalize_or_zero();
        let phase = t * platform.translation_speed + platform.phase;
        let displacement = axis * (phase.sin() * platform.translation_amplitude);
        transform.translation = platform.origin + displacement;
        linear_velocity.0 =
            axis * (phase.cos() * platform.translation_amplitude * platform.translation_speed);
        transform.rotation = Quat::from_rotation_y(t * platform.rotation_speed);
        angular_velocity.0 = Vec3::Y * platform.rotation_speed;
    }
}

fn spawn_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    color: Color,
) {
    commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn spawn_dynamic_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    color: Color,
) {
    commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.82,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Dynamic,
        Collider::cuboid(size.x, size.y, size.z),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
    ));
}

fn spawn_ramp(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    tilt_z: f32,
    color: Color,
    surface: Option<MovementSurface>,
) {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.84,
            ..default()
        })),
        Transform::from_translation(center).with_rotation(Quat::from_rotation_z(tilt_z)),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
    if let Some(surface) = surface {
        entity.insert(surface);
    }
}

fn spawn_stairs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    start: Vec3,
    steps: usize,
    depth: f32,
    height: f32,
    width: f32,
) {
    for step in 0..steps {
        let size = Vec3::new(width, height * (step as f32 + 1.0), depth);
        let center = start + Vec3::new(0.0, size.y * 0.5, -(step as f32 * depth) - depth * 0.5);
        spawn_block(
            commands,
            meshes,
            materials,
            &format!("Lab Step {step}"),
            center,
            size,
            Color::srgb(0.34, 0.41, 0.52),
        );
    }
}

fn spawn_platform(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    color: Color,
    moving: Option<MovingPlatform>,
    surface: Option<MovementSurface>,
) {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.05,
            perceptual_roughness: 0.75,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Kinematic,
        Collider::cuboid(size.x, size.y, size.z),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
    ));
    if let Some(moving) = moving {
        entity.insert(moving);
    }
    if let Some(surface) = surface {
        entity.insert(surface);
    }
}

fn spawn_water_volume(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
) {
    commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.18, 0.46, 0.72, 0.38),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.2,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
        WaterVolume {
            speed_multiplier: 0.55,
            acceleration_multiplier: 0.8,
            gravity_multiplier: 0.4,
        },
    ));
}
