//! # Basic Character Controller
//!
//! The simplest working character controller: walk, sprint, jump, and crouch on a flat
//! ground plane with platforms at varying heights to verify jumping.
//!
//! **Demonstrates**: plugin setup, character entity spawn, input wiring, FPS camera integration.
//! **Test**: walk on ground, jump onto platforms, sprint, crouch.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, add_demo_controller_plugins, add_diagnostic_hud,
    animate_platforms, default_character_actions, spawn_block, spawn_demo_instructions,
    spawn_flat_ground, spawn_fps_camera, spawn_lighting,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerSystems, CharacterFlying, CharacterPush,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller basic");

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
        "Basic Controller",
        &[
            "Walk on the ground, jump onto the colored platforms.",
            "Check the HUD (bottom-left) to verify grounded state.",
        ],
    );

    // -- Jump test platforms at increasing heights ----------------------------
    // Low platform (easy jump)
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Low Platform",
        Vec3::new(4.0, 0.4, 0.0),
        Vec3::new(3.0, 0.8, 3.0),
        Color::srgb(0.35, 0.58, 0.44),
    );
    // Medium platform
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Medium Platform",
        Vec3::new(4.0, 0.9, -5.0),
        Vec3::new(3.0, 1.8, 3.0),
        Color::srgb(0.56, 0.49, 0.26),
    );
    // Tall wall (cannot jump over — tests collision)
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Tall Wall",
        Vec3::new(-5.0, 2.0, -4.0),
        Vec3::new(6.0, 4.0, 0.6),
        Color::srgb(0.62, 0.32, 0.28),
    );
    // Gap jump target — narrow platform across a gap
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Gap Target",
        Vec3::new(-5.0, 0.3, 4.0),
        Vec3::new(2.0, 0.6, 2.0),
        Color::srgb(0.39, 0.47, 0.62),
    );

    // -- Player character ---------------------------------------------------
    let player_transform = Transform::from_xyz(0.0, 3.0, 10.0);

    let controller = CharacterController {
        speed: 11.0,
        jump_input_buffer: Duration::from_millis(160),
        coyote_time: Duration::from_millis(110),
        ..default()
    };

    commands.spawn((
        Name::new("Player"),
        DemoPlayer,
        controller,
        CharacterFlying::default(),
        CharacterPush::default(),
        Visibility::Inherited,
        player_transform,
        default_character_actions(),
    ));

    // -- FPS camera (driven by saddle-camera-fps-camera) --------------------
    spawn_fps_camera(&mut commands, &player_transform);
}
