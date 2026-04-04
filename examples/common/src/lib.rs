//! Shared utilities for `saddle-character-controller` examples.
//!
//! This module provides **non-library-specific** boilerplate: window setup, basic 3D scene
//! scaffolding, geometry helpers, camera follow rigs, cursor management, and saddle-pane
//! integration. Each example's `main.rs` is responsible for plugin registration, character
//! spawning, input wiring, and scene-specific geometry.

use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, WindowResolution},
};
use saddle_character_controller::{
    CharacterController, CharacterControllerState, CharacterFlying, CharacterLook,
    CharacterMotionStats, FlightCollisionMode, SupportRotationPolicy,
};
use saddle_pane::prelude::*;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the player-controlled character entity. Used by the pane sync system.
#[derive(Component)]
pub struct DemoPlayer;

// ---------------------------------------------------------------------------
// Camera components & systems
// ---------------------------------------------------------------------------

/// Attach to a `Camera3d` entity for first-person follow behavior.
#[derive(Component)]
pub struct FirstPersonCamera {
    pub target: Entity,
}

/// Attach to a `Camera3d` entity for third-person follow behavior.
#[derive(Component)]
pub struct ThirdPersonCamera {
    pub target: Entity,
    pub distance: f32,
    pub height: f32,
}

pub fn follow_first_person_camera(
    targets: Query<
        (&Transform, &CharacterController, &CharacterControllerState),
        Without<FirstPersonCamera>,
    >,
    mut cameras: Query<(&FirstPersonCamera, &mut Transform), Without<CharacterController>>,
) {
    for (camera, mut transform) in &mut cameras {
        let Ok((target_transform, controller, state)) = targets.get(camera.target) else {
            continue;
        };
        let view_height = if state.crouching {
            controller.crouch_view_height
        } else {
            controller.standing_view_height
        };
        transform.translation = target_transform.translation + Vec3::Y * view_height;
        transform.rotation = state.orientation;
    }
}

pub fn follow_third_person_camera(
    targets: Query<
        (&Transform, &CharacterControllerState),
        (With<CharacterController>, Without<ThirdPersonCamera>),
    >,
    mut cameras: Query<(&ThirdPersonCamera, &mut Transform), Without<CharacterController>>,
) {
    for (camera, mut transform) in &mut cameras {
        let Ok((target_transform, state)) = targets.get(camera.target) else {
            continue;
        };
        let focus = target_transform.translation + Vec3::Y * camera.height * 0.65;
        let backward = state.orientation * Vec3::Z;
        transform.translation =
            focus + backward * camera.distance + Vec3::Y * (camera.height * 0.55);
        *transform = transform.looking_at(focus, Vec3::Y);
    }
}

// ---------------------------------------------------------------------------
// Cursor management
// ---------------------------------------------------------------------------

pub fn capture_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

pub fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::None;
    cursor.visible = true;
}

/// Adds cursor capture/release systems (click to grab, Escape to release).
pub fn add_cursor_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
            release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
        ),
    );
}

// ---------------------------------------------------------------------------
// Saddle-pane integration
// ---------------------------------------------------------------------------

pub fn pane_plugins() -> (
    bevy_flair::FlairPlugin,
    bevy_input_focus::InputDispatchPlugin,
    bevy_ui_widgets::UiWidgetsPlugins,
    bevy_input_focus::tab_navigation::TabNavigationPlugin,
    saddle_pane::PanePlugin,
) {
    (
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        saddle_pane::PanePlugin,
    )
}

#[derive(Resource, Pane)]
#[pane(title = "Character Controller")]
pub struct ControllerPane {
    #[pane(tab = "Movement", slider, min = 4.0, max = 20.0, step = 0.1)]
    pub speed: f32,
    #[pane(tab = "Movement", slider, min = 1.0, max = 2.5, step = 0.05)]
    pub sprint_speed_scale: f32,
    #[pane(tab = "Movement", slider, min = 0.5, max = 3.0, step = 0.05)]
    pub jump_height: f32,
    #[pane(tab = "Movement", slider, min = 0.1, max = 1.2, step = 0.05)]
    pub step_size: f32,
    #[pane(tab = "Movement", slider, min = 0.0, max = 24.0, step = 0.5)]
    pub air_acceleration_hz: f32,
    #[pane(tab = "Movement", slider, min = 0.0005, max = 0.01, step = 0.0001)]
    pub look_sensitivity: f32,
    #[pane(tab = "Movement")]
    pub inherit_support_yaw: bool,
    #[pane(tab = "Traversal")]
    pub flight_enabled: bool,
    #[pane(tab = "Traversal", slider, min = 4.0, max = 30.0, step = 0.5)]
    pub flight_speed: f32,
    #[pane(tab = "Traversal")]
    pub flight_noclip: bool,
    #[pane(tab = "Runtime", monitor)]
    pub movement_mode: String,
    #[pane(tab = "Runtime", monitor)]
    pub grounded: bool,
    #[pane(tab = "Runtime", monitor)]
    pub current_speed: f32,
    #[pane(tab = "Runtime", monitor)]
    pub support_entity: String,
}

impl Default for ControllerPane {
    fn default() -> Self {
        Self {
            speed: 11.0,
            sprint_speed_scale: 1.5,
            jump_height: 1.8,
            step_size: 0.7,
            air_acceleration_hz: 12.0,
            look_sensitivity: 0.0022,
            inherit_support_yaw: true,
            flight_enabled: false,
            flight_speed: 14.0,
            flight_noclip: false,
            movement_mode: "Airborne".into(),
            grounded: false,
            current_speed: 0.0,
            support_entity: "None".into(),
        }
    }
}

/// Bidirectional sync between [`ControllerPane`] and the player's controller components.
/// Register this as an `Update` system.
pub fn sync_controller_pane(
    mut pane: ResMut<ControllerPane>,
    mut players: Query<
        (
            &mut CharacterController,
            &mut CharacterLook,
            &CharacterControllerState,
            &CharacterMotionStats,
            Option<&mut CharacterFlying>,
        ),
        With<DemoPlayer>,
    >,
) {
    let Ok((mut controller, mut look, state, stats, flying)) = players.single_mut() else {
        return;
    };
    let mut flight_snapshot = None;

    if pane.is_changed() && !pane.is_added() {
        controller.speed = pane.speed;
        controller.sprint_speed_scale = pane.sprint_speed_scale;
        controller.jump_height = pane.jump_height;
        controller.step_size = pane.step_size;
        controller.air_acceleration_hz = pane.air_acceleration_hz;
        controller.support_rotation_policy = if pane.inherit_support_yaw {
            SupportRotationPolicy::YawOnly
        } else {
            SupportRotationPolicy::None
        };
        look.sensitivity = Vec2::splat(pane.look_sensitivity);
    }

    if let Some(mut flying) = flying {
        if pane.is_changed() && !pane.is_added() {
            flying.enabled = pane.flight_enabled;
            flying.speed = pane.flight_speed;
            flying.collision_mode = if pane.flight_noclip {
                FlightCollisionMode::NoClip
            } else {
                FlightCollisionMode::Slide
            };
        }
        flight_snapshot = Some((
            flying.enabled,
            flying.speed,
            matches!(flying.collision_mode, FlightCollisionMode::NoClip),
        ));
    }

    let pane = pane.bypass_change_detection();
    pane.speed = controller.speed;
    pane.sprint_speed_scale = controller.sprint_speed_scale;
    pane.jump_height = controller.jump_height;
    pane.step_size = controller.step_size;
    pane.air_acceleration_hz = controller.air_acceleration_hz;
    pane.look_sensitivity = look.sensitivity.x;
    pane.inherit_support_yaw =
        controller.support_rotation_policy == SupportRotationPolicy::YawOnly;
    if let Some((enabled, speed, noclip)) = flight_snapshot {
        pane.flight_enabled = enabled;
        pane.flight_speed = speed;
        pane.flight_noclip = noclip;
    } else {
        pane.flight_enabled = false;
        pane.flight_speed = 0.0;
        pane.flight_noclip = false;
    }
    pane.movement_mode = format!("{:?}", state.movement_mode);
    pane.grounded = state.ground.is_some();
    pane.current_speed = stats.current_speed;
    pane.support_entity = stats
        .last_support_entity
        .map(|entity| format!("{entity:?}"))
        .unwrap_or_else(|| "None".into());
}

// ---------------------------------------------------------------------------
// Moving-platform support (used by multiple examples)
// ---------------------------------------------------------------------------

/// Attach to a kinematic rigid-body to make it oscillate and/or rotate each fixed tick.
#[derive(Component)]
pub struct MovingPlatform {
    pub origin: Vec3,
    pub translation_axis: Vec3,
    pub translation_amplitude: f32,
    pub translation_speed: f32,
    pub rotation_speed: f32,
    pub phase: f32,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemoFixedSystems {
    AnimatePlatforms,
}

pub fn animate_platforms(
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

// ---------------------------------------------------------------------------
// Scene helpers — lighting, ground, geometry primitives
// ---------------------------------------------------------------------------

pub fn spawn_lighting(commands: &mut Commands) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 24_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(16.0, 22.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn spawn_flat_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    size: f32,
) {
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(size)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.17, 0.21, 0.19),
            perceptual_roughness: 1.0,
            ..default()
        })),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));
}

pub fn spawn_block(
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
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_ramp(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    tilt_z: f32,
    color: Color,
    surface: Option<saddle_character_controller::MovementSurface>,
) {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.85,
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

#[allow(clippy::too_many_arguments)]
pub fn spawn_stairs(
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
            &format!("Step {step}"),
            center,
            size,
            Color::srgb(0.34, 0.41, 0.52),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_platform(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    color: Color,
    moving: Option<MovingPlatform>,
    surface: Option<saddle_character_controller::MovementSurface>,
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

pub fn spawn_water_volume(
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
        saddle_character_controller::WaterVolume::default(),
    ));
}

/// Spawn a visible capsule-shaped body as a child of the given entity.
pub fn spawn_controller_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    entity: Entity,
    color: Color,
) {
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 1.8, 0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.85,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.9, 0.0),
        ));
    });
}

/// Create an [`App`] pre-configured with the window, physics, pane plugins, and common systems.
/// The caller still needs to add `CharacterControllerPlugin`, scene setup, and camera systems.
pub fn base_app(title: &str) -> App {
    let mut app = App::new();
    app.insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: title.into(),
                    resolution: WindowResolution::new(1440, 900),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            pane_plugins(),
        ))
        .register_pane::<ControllerPane>();
    add_cursor_systems(&mut app);
    app.add_systems(Update, sync_controller_pane);
    app
}
