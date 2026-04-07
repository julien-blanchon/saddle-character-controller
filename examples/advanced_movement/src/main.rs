//! # Advanced Movement
//!
//! Source-engine-style movement: bunny-hopping, air-strafing, and surfing.
//! The controller is tuned for fast-paced movement with auto_bhop and generous air
//! acceleration.
//!
//! **Demonstrates**: `auto_bhop`, `air_acceleration_hz`, `max_air_wish_speed`,
//! `slide_gravity_scale`, `MovementSurface` (surf walls).
//! **Test**: hold space to bhop, air-strafe on surf walls, ride conveyor.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, MovingPlatform, add_demo_controller_plugins, add_diagnostic_hud,
    animate_platforms, default_character_actions, spawn_block, spawn_demo_instructions,
    spawn_flat_ground, spawn_fps_camera, spawn_lighting, spawn_platform, spawn_ramp,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerDebugDraw, CharacterControllerSystems, CharacterFlying,
    CharacterPush, MovementSurface, SupportVelocityPolicy,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller advanced_movement");

    add_demo_controller_plugins(&mut app);
    add_diagnostic_hud(&mut app);

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
    );

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
            "Hold SPACE to bunny-hop continuously.",
            "RED/BLUE angled walls: surf by air-strafing (A/D).",
            "PURPLE strip: conveyor belt pushes you sideways.",
            "ORANGE platform: rides along X axis.",
            "Watch HUD speed while surfing.",
        ],
    );

    // -- Surf ramp pair (V-shape, thick enough to not tunnel through) --------
    // Left wall: tilted ~50 degrees, thick body for reliable collision.
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Surf Wall Left",
        Vec3::new(20.0, 2.5, -8.0),
        Vec3::new(16.0, 2.0, 10.0),
        -0.9,
        Color::srgb(0.76, 0.36, 0.28),
        Some(MovementSurface {
            slide_only: true,
            traction_multiplier: 0.15,
            ..default()
        }),
    );
    // Right wall
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Surf Wall Right",
        Vec3::new(20.0, 2.5, 8.0),
        Vec3::new(16.0, 2.0, 10.0),
        0.9,
        Color::srgb(0.28, 0.55, 0.74),
        Some(MovementSurface {
            slide_only: true,
            traction_multiplier: 0.15,
            ..default()
        }),
    );

    // -- Launch ramp to reach surf walls ------------------------------------
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Launch Ramp",
        Vec3::new(10.0, 0.6, 0.0),
        Vec3::new(6.0, 0.6, 5.0),
        -0.35,
        Color::srgb(0.4, 0.6, 0.3),
        None,
    );

    // -- Moving platform (along X) ------------------------------------------
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Slider Platform",
        Vec3::new(-10.0, 1.0, 0.0),
        Vec3::new(4.0, 0.35, 4.0),
        Color::srgb(0.82, 0.57, 0.2),
        Some(MovingPlatform {
            origin: Vec3::new(-10.0, 1.0, 0.0),
            translation_axis: Vec3::X,
            translation_amplitude: 6.0,
            translation_speed: 0.8,
            rotation_speed: 0.0,
            phase: 0.0,
        }),
        None,
    );

    // -- Conveyor strip -----------------------------------------------------
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Conveyor Strip",
        Vec3::new(-6.0, 0.3, -10.0),
        Vec3::new(8.0, 0.2, 3.0),
        Color::srgb(0.42, 0.37, 0.76),
        None,
        Some(MovementSurface {
            conveyor_velocity: Vec3::new(3.0, 0.0, 0.0),
            inherit_velocity_policy: Some(SupportVelocityPolicy::Horizontal),
            ..default()
        }),
    );

    // -- Bhop practice blocks -----------------------------------------------
    for i in 0..5 {
        spawn_block(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Bhop Block {i}"),
            Vec3::new(-20.0 + i as f32 * 4.0, 0.25, -5.0),
            Vec3::new(2.0, 0.5, 2.0),
            Color::srgb(0.5, 0.5, 0.3),
        );
    }

    // -- Player character (tuned for advanced movement) ----------------------
    let player_transform = Transform::from_xyz(-20.0, 3.0, 6.0);

    commands.spawn((
        Name::new("Player"),
        DemoPlayer,
        CharacterController {
            speed: 14.0,
            jump_input_buffer: Duration::from_millis(160),
            coyote_time: Duration::from_millis(110),
            friction_hz: 7.0,
            air_acceleration_hz: 18.0,
            max_air_wish_speed: 0.95,
            auto_bhop: true,
            slide_gravity_scale: 1.2,
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
