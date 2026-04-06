//! # Basic Character Controller
//!
//! The simplest possible working character controller: a capsule that walks, sprints,
//! crouches, and jumps on a flat ground plane with a few static obstacles.
//!
//! **Demonstrates**: plugin setup, character entity spawn, input wiring, FPS camera integration.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, add_demo_controller_plugins, animate_platforms,
    default_character_actions, spawn_block, spawn_demo_instructions, spawn_flat_ground,
    spawn_fps_camera, spawn_lighting,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerSystems, CharacterFlying, CharacterPush,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller basic");

    add_demo_controller_plugins(&mut app);

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
        &["Explore the crate field, then use the pane to tweak speed, jump height, and flight."],
    );

    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Crate Cluster A",
        Vec3::new(-4.0, 1.0, 2.0),
        Vec3::new(2.0, 2.0, 2.0),
        Color::srgb(0.56, 0.39, 0.26),
    );
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Crate Cluster B",
        Vec3::new(4.5, 0.5, -3.0),
        Vec3::new(3.0, 1.0, 3.0),
        Color::srgb(0.39, 0.47, 0.62),
    );
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Crate Cluster C",
        Vec3::new(0.0, 1.5, -10.0),
        Vec3::new(2.5, 3.0, 2.5),
        Color::srgb(0.31, 0.58, 0.47),
    );

    // -- Player character ---------------------------------------------------
    let player_transform = Transform::from_xyz(0.0, 3.0, 14.0);

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
