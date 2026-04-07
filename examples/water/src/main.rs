//! # Water / Swimming
//!
//! Shows the character controller transitioning between walking and swimming when
//! entering a `SwimVolume` trigger. The controller automatically switches to swim
//! mode: buoyancy, reduced gravity, and different movement feel.
//!
//! **Demonstrates**: `SwimVolume`, `CharacterSwimming`, `AscendAction` (Q key).
//! **Test**: walk into the pool, swim with WASD, ascend with Q, exit pool.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, add_demo_controller_plugins, add_diagnostic_hud,
    animate_platforms, default_character_actions, spawn_block, spawn_demo_instructions,
    spawn_flat_ground, spawn_fps_camera, spawn_lighting, spawn_water_volume,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerSystems, CharacterFlying, CharacterPush,
    CharacterSwimming,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller water");

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
        "Water / Swimming",
        &[
            "Walk forward into the BLUE pool.",
            "WASD to swim, Q to ascend.",
            "Watch HUD mode change to Swimming.",
            "Swim out the far side to return to ground.",
        ],
    );

    // -- Pool structure: raised rim around a water volume --------------------
    // Left wall
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Pool Wall Left",
        Vec3::new(-5.5, 0.4, -6.0),
        Vec3::new(0.5, 0.8, 10.0),
        Color::srgb(0.3, 0.35, 0.4),
    );
    // Right wall
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Pool Wall Right",
        Vec3::new(5.5, 0.4, -6.0),
        Vec3::new(0.5, 0.8, 10.0),
        Color::srgb(0.3, 0.35, 0.4),
    );
    // Far wall
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Pool Wall Far",
        Vec3::new(0.0, 0.4, -11.5),
        Vec3::new(11.5, 0.8, 0.5),
        Color::srgb(0.3, 0.35, 0.4),
    );
    // Near wall (lower, so you can walk in)
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Pool Wall Near",
        Vec3::new(0.0, 0.15, -0.5),
        Vec3::new(11.5, 0.3, 0.5),
        Color::srgb(0.3, 0.35, 0.4),
    );

    // Water trigger volume (fills the pool interior)
    spawn_water_volume(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Pool Water",
        Vec3::new(0.0, 0.8, -6.0),
        Vec3::new(10.0, 3.0, 10.0),
    );

    // -- Player character ---------------------------------------------------
    let player_transform = Transform::from_xyz(0.0, 3.0, 6.0);

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
        CharacterSwimming::default(),
        Visibility::Inherited,
        player_transform,
        default_character_actions(),
    ));

    spawn_fps_camera(&mut commands, &player_transform);
}
