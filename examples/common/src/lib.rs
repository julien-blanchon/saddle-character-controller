//! Shared utilities for `saddle-character-controller` examples.
//!
//! This module provides **non-library-specific** boilerplate: window setup, basic 3D scene
//! scaffolding, geometry helpers, camera follow rigs, cursor management, and saddle-pane
//! integration. Each example's `main.rs` is responsible for plugin registration, character
//! spawning, input wiring, and scene-specific geometry.

use avian3d::prelude::*;
use bevy::{
    input::{common_conditions::input_just_pressed, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, WindowResolution},
};
use bevy_enhanced_input::prelude::*;
use saddle_character_controller::{
    AccumulatedInput, CharacterController, CharacterControllerPlugin, CharacterControllerState,
    CharacterFlying, CharacterMotionStats, FlightCollisionMode, SupportRotationPolicy,
    adapters::enhanced_input::{
        AscendAction, CharacterControllerEnhancedInputPlugin, CrouchAction, JumpAction,
        MoveAction, SprintAction, TraverseAction,
    },
    convenience::environment::{CharacterControllerEnvironmentPlugin, SwimVolume},
};
use saddle_camera_fps_camera::{
    FpsCamera, FpsCameraConfig, FpsCameraEffectsPlugin, FpsCameraExternalMotion, FpsCameraIntent,
    FpsCameraPlugin, FpsCameraRuntime, FpsCameraSystems,
};
use saddle_pane::prelude::*;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the player-controlled character entity. Used by the pane sync system.
#[derive(Component)]
pub struct DemoPlayer;

#[derive(Component)]
pub struct DemoInstructions;

// ---------------------------------------------------------------------------
// FPS camera bridge (saddle-camera-fps-camera ↔ character controller)
// ---------------------------------------------------------------------------

/// Feeds character controller state into [`FpsCameraExternalMotion`] each frame.
pub fn sync_fps_camera_motion(
    player: Query<
        (
            &Transform,
            &LinearVelocity,
            &CharacterController,
            &CharacterControllerState,
        ),
        With<DemoPlayer>,
    >,
    mut camera: Query<&mut FpsCameraExternalMotion, With<FpsCamera>>,
) {
    let Ok((transform, velocity, controller, state)) = player.single() else {
        return;
    };
    let Ok(mut ext) = camera.single_mut() else {
        return;
    };
    let view_height = if state.crouching {
        controller.crouch_view_height
    } else {
        controller.standing_view_height
    };
    ext.enabled = true;
    ext.position = transform.translation;
    ext.velocity = velocity.0;
    ext.grounded = state.ground.is_some();
    ext.eye_height = Some(view_height);
    ext.crouch_alpha = Some(if state.crouching { 1.0 } else { 0.0 });
    ext.sprint_alpha = Some(
        (velocity.0.xz().length() / (controller.speed * controller.sprint_speed_scale))
            .clamp(0.0, 1.0),
    );
}

/// Copies FPS camera orientation → controller state so movement is aligned with camera look.
pub fn sync_fps_camera_look(
    mut player: Query<&mut CharacterControllerState, With<DemoPlayer>>,
    camera: Query<&FpsCameraRuntime, With<FpsCamera>>,
) {
    let Ok(runtime) = camera.single() else {
        return;
    };
    let Ok(mut state) = player.single_mut() else {
        return;
    };
    state.orientation = Quat::from_euler(EulerRot::YXZ, runtime.yaw, runtime.pitch, 0.0);
}

/// Feeds mouse motion into the FPS camera's look intent, gated by cursor lock state.
pub fn feed_fps_camera_look(
    lock_state: Res<CursorLockState>,
    mouse: Res<AccumulatedMouseMotion>,
    mut camera: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if !lock_state.0 {
        return;
    }
    let Ok(mut intent) = camera.single_mut() else {
        return;
    };
    intent.look_delta += mouse.delta;
}

/// Registers the FPS camera plugins and bridge systems with correct ordering.
/// Call this instead of manually adding camera follow systems.
pub fn add_fps_camera_bridge(app: &mut App) {
    app.add_plugins((FpsCameraPlugin::default(), FpsCameraEffectsPlugin::default()));
    app.add_systems(
        Update,
        (
            feed_fps_camera_look.before(FpsCameraSystems::ReadIntent),
            sync_fps_camera_motion.before(FpsCameraSystems::UpdateLocomotion),
            sync_fps_camera_look
                .after(FpsCameraSystems::ReadIntent)
                .before(FpsCameraSystems::UpdateLocomotion),
        ),
    );
}

/// Spawns a first-person camera entity driven by [`FpsCameraExternalMotion`].
/// Returns the camera entity.
pub fn spawn_fps_camera(commands: &mut Commands, player_transform: &Transform) -> Entity {
    let config = FpsCameraConfig::default();
    commands
        .spawn((
            Name::new("FPS Camera"),
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: config.fov.base_fov,
                ..default()
            }),
            *player_transform,
            FpsCamera,
            config,
            FpsCameraExternalMotion {
                enabled: true,
                ..default()
            },
        ))
        .id()
}

// ---------------------------------------------------------------------------
// Legacy camera components (kept for third-person example)
// ---------------------------------------------------------------------------

/// Attach to a `Camera3d` entity for third-person follow behavior.
#[derive(Component)]
pub struct ThirdPersonCamera {
    pub target: Entity,
    pub distance: f32,
    pub height: f32,
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

/// Tracks whether camera look is active. Decoupled from OS cursor state to avoid
/// platform-dependent issues (e.g. macOS not locking cursor when window lacks focus).
#[derive(Resource)]
pub struct CursorLockState(pub bool);

fn capture_cursor(mut cursor: Single<&mut CursorOptions>, mut state: ResMut<CursorLockState>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
    state.0 = true;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>, mut state: ResMut<CursorLockState>) {
    cursor.grab_mode = CursorGrabMode::None;
    cursor.visible = true;
    state.0 = false;
}

fn no_ui_hovered(query: Query<&Interaction>) -> bool {
    query.iter().all(|i| matches!(i, Interaction::None))
}

/// Adds cursor capture/release systems (click to grab, Escape to release).
/// Camera look is active by default. Press Escape to free the cursor and freeze look,
/// then click to recapture.
pub fn add_cursor_systems(app: &mut App) {
    app.insert_resource(CursorLockState(true));
    app.add_systems(Startup, capture_cursor);
    app.add_systems(
        Update,
        (
            capture_cursor
                .run_if(input_just_pressed(MouseButton::Left).and(no_ui_hovered)),
            release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
        )
            .chain(),
    );
}

pub fn add_demo_controller_plugins(app: &mut App) {
    app.add_plugins((
        CharacterControllerPlugin::always_on(FixedUpdate),
        CharacterControllerEnhancedInputPlugin,
        CharacterControllerEnvironmentPlugin::new(FixedUpdate),
    ));
}

pub fn default_character_actions() -> impl Bundle {
    actions!(CharacterController[
        (
            Action::<MoveAction>::new(),
            DeadZone::default(),
            Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
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
    ])
}

pub fn spawn_demo_instructions(commands: &mut Commands, title: &str, extra_lines: &[&str]) {
    let mut text = String::new();
    text.push_str(title);
    text.push_str("\nWASD / Left Stick: move\nMouse / Right Stick: look\nSpace: jump\nShift: sprint\nCtrl or C: crouch\nQ: ascend when flying or swimming\nE: traversal ability\nPane: toggle flight and tuning\nLeft Click: lock cursor\nEsc: release cursor");
    for line in extra_lines {
        text.push('\n');
        text.push_str(line);
    }

    commands.spawn((
        Name::new(format!("{title} Instructions")),
        DemoInstructions,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Px(340.0),
            padding: UiRect::all(Val::Px(14.0)),
            border_radius: BorderRadius::all(Val::Px(16.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.82)),
        Text::new(text),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
        ZIndex(10),
    ));
}

// ---------------------------------------------------------------------------
// Diagnostic HUD — real-time controller state overlay (bottom-left)
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct DiagnosticHud;

pub fn spawn_diagnostic_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("Diagnostic HUD"),
        DiagnosticHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            width: Val::Px(380.0),
            padding: UiRect::all(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.85)),
        Text::new("Loading..."),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.9, 0.7)),
        ZIndex(10),
    ));
}

pub fn update_diagnostic_hud(
    players: Query<
        (
            &Transform,
            &LinearVelocity,
            &CharacterControllerState,
            &CharacterMotionStats,
            &AccumulatedInput,
        ),
        With<DemoPlayer>,
    >,
    mut huds: Query<&mut Text, With<DiagnosticHud>>,
) {
    let Ok((transform, velocity, state, stats, input)) = players.single() else {
        return;
    };
    let Ok(mut text) = huds.single_mut() else {
        return;
    };
    let grounded_str = if let Some(ground) = state.ground {
        if ground.walkable {
            format!("YES (n={:.2},{:.2},{:.2})", ground.normal.x, ground.normal.y, ground.normal.z)
        } else {
            "SLIDING".to_string()
        }
    } else {
        "NO".to_string()
    };
    let jump_buf = input
        .jump_pressed_for
        .map(|age| format!("{:.0}ms", age * 1000.0))
        .unwrap_or_else(|| "-".to_string());
    **text = format!(
        "Mode: {:?}\n\
         Grounded: {}\n\
         Position: ({:.1}, {:.1}, {:.1})\n\
         Velocity: ({:.1}, {:.1}, {:.1})\n\
         H.Speed: {:.1}  V.Speed: {:.1}\n\
         Speed: {:.1}\n\
         Jump buffer: {}\n\
         Crouching: {}\n\
         Casts/tick: {}",
        state.movement_mode,
        grounded_str,
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
        velocity.x,
        velocity.y,
        velocity.z,
        stats.horizontal_speed,
        velocity.y,
        stats.current_speed,
        jump_buf,
        state.crouching,
        stats.shape_casts_last_tick,
    );
}

/// Adds diagnostic HUD spawn + update systems. Call after `base_app` or manually.
pub fn add_diagnostic_hud(app: &mut App) {
    app.add_systems(Startup, spawn_diagnostic_hud);
    app.add_systems(Update, update_diagnostic_hud);
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
            &CharacterControllerState,
            &CharacterMotionStats,
            Option<&mut CharacterFlying>,
        ),
        With<DemoPlayer>,
    >,
) {
    let Ok((mut controller, state, stats, flying)) = players.single_mut() else {
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
    pane.inherit_support_yaw = controller.support_rotation_policy == SupportRotationPolicy::YawOnly;
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
    // Use a thick cuboid instead of half_space — Avian3D's shape cast (used for ground
    // probing) is unreliable against infinite half-space colliders, causing the controller
    // to never detect ground and thus never allow jumping.
    let thickness = 1.0;
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(size)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.17, 0.21, 0.19),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -thickness * 0.5, 0.0),
        RigidBody::Static,
        Collider::cuboid(size * 2.0, thickness, size * 2.0),
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
        SwimVolume::default(),
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

/// Create an [`App`] pre-configured with the window, physics, FPS camera bridge, pane,
/// and cursor management. The caller still needs to add scene setup and spawn the player +
/// camera (use [`spawn_fps_camera`]).
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
    add_fps_camera_bridge(&mut app);
    add_cursor_systems(&mut app);
    app.add_systems(Update, sync_controller_pane);
    app
}
