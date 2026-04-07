//! # Moving Platforms
//!
//! Shows the character controller riding kinematic platforms that translate and rotate.
//! The controller inherits platform velocity via `SupportVelocityPolicy`, and a conveyor
//! belt shows `MovementSurface::conveyor_velocity`.
//!
//! **Demonstrates**: `SupportVelocityPolicy`, `support_detach_grace`, conveyor surfaces.
//! **Test**: ride platforms, jump off (inherit velocity), stand on conveyor.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, MovingPlatform, add_demo_controller_plugins, add_diagnostic_hud,
    animate_platforms, default_character_actions, spawn_demo_instructions, spawn_flat_ground,
    spawn_fps_camera, spawn_lighting, spawn_platform,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerSystems, CharacterFlying, CharacterPush,
    MovementSurface, SupportVelocityPolicy,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller moving_platforms");

    add_demo_controller_plugins(&mut app);
    add_diagnostic_hud(&mut app);

    app.configure_sets(
        FixedUpdate,
        DemoFixedSystems::AnimatePlatforms.before(CharacterControllerSystems::Grounding),
    )
    .add_systems(Startup, setup_scene)
    .add_systems(
        FixedUpdate,
        animate_platforms.in_set(DemoFixedSystems::AnimatePlatforms),
    );

    app.run()
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_lighting(&mut commands);
    spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 90.0);
    spawn_demo_instructions(
        &mut commands,
        "Moving Platforms",
        &[
            "ORANGE: slides along X (horizontal inherit).",
            "BLUE: slides along Z (full velocity inherit).",
            "PURPLE: conveyor belt (pushes you sideways).",
            "Jump off platforms to test velocity inheritance.",
        ],
    );

    // -- Platform A: horizontal slider (default policy = Horizontal) ---------
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Platform A (X-slide)",
        Vec3::new(-6.0, 1.0, 0.0),
        Vec3::new(4.0, 0.4, 4.0),
        Color::srgb(0.82, 0.57, 0.2),
        Some(MovingPlatform {
            origin: Vec3::new(-6.0, 1.0, 0.0),
            translation_axis: Vec3::X,
            translation_amplitude: 5.0,
            translation_speed: 0.9,
            rotation_speed: 0.0,
            phase: 0.0,
        }),
        None,
    );

    // -- Platform B: Z-slider with full velocity inheritance -----------------
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Platform B (Z-slide, Full)",
        Vec3::new(6.0, 1.0, 0.0),
        Vec3::new(4.0, 0.4, 4.0),
        Color::srgb(0.31, 0.52, 0.84),
        Some(MovingPlatform {
            origin: Vec3::new(6.0, 1.0, 0.0),
            translation_axis: Vec3::Z,
            translation_amplitude: 5.0,
            translation_speed: 1.0,
            rotation_speed: 0.0,
            phase: 1.0,
        }),
        Some(MovementSurface {
            inherit_velocity_policy: Some(SupportVelocityPolicy::Full),
            ..default()
        }),
    );

    // -- Conveyor belt (static platform with surface velocity) ---------------
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Conveyor Belt",
        Vec3::new(0.0, 0.3, -8.0),
        Vec3::new(8.0, 0.2, 3.0),
        Color::srgb(0.42, 0.37, 0.76),
        None,
        Some(MovementSurface {
            conveyor_velocity: Vec3::new(3.0, 0.0, 0.0),
            inherit_velocity_policy: Some(SupportVelocityPolicy::Horizontal),
            ..default()
        }),
    );

    // -- Player character ---------------------------------------------------
    let player_transform = Transform::from_xyz(0.0, 3.0, 10.0);

    commands.spawn((
        Name::new("Player"),
        DemoPlayer,
        CharacterController {
            speed: 11.0,
            jump_input_buffer: Duration::from_millis(160),
            coyote_time: Duration::from_millis(110),
            ..default()
        },
        CharacterFlying::default(),
        CharacterPush::default(),
        Visibility::Inherited,
        player_transform,
        default_character_actions(),
    ));

    spawn_fps_camera(&mut commands, &player_transform);
}
