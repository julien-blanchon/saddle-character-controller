//! # Advanced Movement
//!
//! An obstacle course demonstrating higher-level movement mechanics: bunny-hopping,
//! air-strafing, surfing, mantling, and wall-kicking. The controller is tuned for
//! fast-paced Source-engine-style movement.
//!
//! **Demonstrates**: `auto_bhop`, `air_acceleration_hz`, `max_air_wish_speed`,
//! `slide_gravity_scale`, `CharacterMantle`, `CharacterWallKick`, `MovementSurface`
//! (surf walls), debug draw overlay.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, FirstPersonCamera, MovingPlatform, add_demo_controller_plugins,
    animate_platforms, default_character_actions, follow_first_person_camera,
    spawn_demo_instructions, spawn_flat_ground, spawn_lighting, spawn_platform, spawn_ramp,
    spawn_stairs,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerDebugDraw, CharacterControllerSystems, CharacterFlying,
    CharacterLook, CharacterMantle, CharacterPush, CharacterWallKick, MovementSurface,
    SupportVelocityPolicy,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller advanced_movement");
    add_demo_controller_plugins(&mut app);

    // Enable debug draw so the player can see grounding rays, step detection, etc.
    app.insert_resource(CharacterControllerDebugDraw {
        enabled: true,
        ..default()
    });

    app.configure_sets(
        FixedUpdate,
        DemoFixedSystems::AnimatePlatforms.before(CharacterControllerSystems::Grounding),
    )
    .add_systems(Startup, setup_scene)
    .add_systems(
        FixedUpdate,
        animate_platforms.in_set(DemoFixedSystems::AnimatePlatforms),
    )
    .add_systems(PostUpdate, follow_first_person_camera);

    app.run()
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_lighting(&mut commands);
    spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 140.0);
    spawn_demo_instructions(
        &mut commands,
        "Advanced Movement",
        &[
            "Use the surf walls, moving platforms, and traversal course while tuning air acceleration and jump height in the pane.",
        ],
    );

    // -- Slopes & stairs section (shared with slopes_and_stairs) ------------
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Walkable Ramp",
        Vec3::new(-6.0, 1.2, 0.0),
        Vec3::new(8.0, 0.8, 8.0),
        -0.4,
        Color::srgb(0.23, 0.49, 0.41),
        None,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Steep Ramp",
        Vec3::new(9.0, 1.2, 0.0),
        Vec3::new(8.0, 0.8, 8.0),
        -1.0,
        Color::srgb(0.63, 0.28, 0.24),
        Some(MovementSurface {
            slide_only: true,
            ..default()
        }),
    );
    spawn_stairs(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-18.0, 0.0, -5.0),
        7,
        1.2,
        0.25,
        2.0,
    );

    // -- Moving platforms section -------------------------------------------
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Platform A",
        Vec3::new(-8.0, 1.0, 0.0),
        Vec3::new(3.5, 0.35, 3.5),
        Color::srgb(0.82, 0.57, 0.2),
        Some(MovingPlatform {
            origin: Vec3::new(-8.0, 1.0, 0.0),
            translation_axis: Vec3::X,
            translation_amplitude: 5.0,
            translation_speed: 0.9,
            rotation_speed: 0.0,
            phase: 0.0,
        }),
        None,
    );
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Platform B",
        Vec3::new(4.0, 2.2, -3.5),
        Vec3::new(3.0, 0.35, 3.0),
        Color::srgb(0.31, 0.52, 0.84),
        Some(MovingPlatform {
            origin: Vec3::new(4.0, 2.2, -3.5),
            translation_axis: Vec3::Z,
            translation_amplitude: 4.0,
            translation_speed: 1.1,
            rotation_speed: 0.0,
            phase: 1.3,
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
        "Conveyor Strip",
        Vec3::new(16.0, 0.4, 0.0),
        Vec3::new(8.0, 0.2, 3.0),
        Color::srgb(0.42, 0.37, 0.76),
        None,
        Some(MovementSurface {
            conveyor_velocity: Vec3::new(2.5, 0.0, 0.0),
            inherit_velocity_policy: Some(SupportVelocityPolicy::Horizontal),
            ..default()
        }),
    );

    // -- Surf walls: steep angled surfaces with low traction ----------------
    // The player can gain speed by air-strafing along these walls — a classic "surf"
    // mechanic. `traction_multiplier` is set low so the controller cannot grip the wall.
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Surf Wall Left",
        Vec3::new(18.0, 2.2, -10.0),
        Vec3::new(14.0, 0.5, 10.0),
        -1.1,
        Color::srgb(0.76, 0.36, 0.28),
        Some(MovementSurface {
            slide_only: true,
            traction_multiplier: 0.25,
            ..default()
        }),
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Surf Wall Right",
        Vec3::new(18.0, 2.2, 10.0),
        Vec3::new(14.0, 0.5, 10.0),
        1.1,
        Color::srgb(0.28, 0.55, 0.74),
        Some(MovementSurface {
            slide_only: true,
            traction_multiplier: 0.25,
            ..default()
        }),
    );

    // -- Player character (tuned for advanced movement) ---------------------
    let controller = CharacterController {
        speed: 14.0,
        jump_input_buffer: Duration::from_millis(160),
        coyote_time: Duration::from_millis(110),
        // Source-engine style: higher friction allows sharper turns on ground
        friction_hz: 7.0,
        // Generous air acceleration enables air-strafing
        air_acceleration_hz: 18.0,
        // Cap wish-speed in air to preserve bunny-hop momentum
        max_air_wish_speed: 0.95,
        // Hold jump to automatically re-jump on landing
        auto_bhop: true,
        // Slide faster on slopes
        slide_gravity_scale: 1.2,
        ..default()
    };
    let look = CharacterLook {
        sensitivity: Vec2::splat(0.0022),
        ..default()
    };

    let player = commands
        .spawn((
            Name::new("Player"),
            DemoPlayer,
            controller,
            look,
            CharacterFlying::default(),
            CharacterPush::default(),
            CharacterMantle::default(),   // grab ledges and pull up
            CharacterWallKick::default(), // kick off walls for extra height
            Visibility::Inherited,
            Transform::from_xyz(-20.0, 3.0, 6.0),
            default_character_actions(),
        ))
        .id();

    commands.spawn((
        Name::new("First Person Camera"),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::TAU / 5.5,
            ..default()
        }),
        FirstPersonCamera { target: player },
    ));
}
