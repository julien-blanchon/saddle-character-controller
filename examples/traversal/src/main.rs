//! # Traversal
//!
//! Parkour-style movement: mantling ledges, wall-kicking, and combining abilities.
//! The course is laid out linearly so each ability is tested one at a time.
//!
//! **Demonstrates**: `CharacterMantle`, `CharacterWallKick`, `step_size`, `coyote_time`.
//! **Test**: jump onto low block, mantle the waist-high block (E while touching it),
//! wall-kick off the tall wall (jump while airborne against it).

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, add_demo_controller_plugins, add_diagnostic_hud,
    animate_platforms, default_character_actions, spawn_block, spawn_demo_instructions,
    spawn_flat_ground, spawn_fps_camera, spawn_lighting, spawn_stairs,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerDebugDraw, CharacterControllerSystems, CharacterFlying,
    CharacterMantle, CharacterPush, CharacterWallKick,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller traversal");

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
    spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 120.0);
    spawn_demo_instructions(
        &mut commands,
        "Traversal Course",
        &[
            "1. BLUE stairs: walk up (auto-step).",
            "2. GREEN block: mantle with E (walk into it + press E).",
            "3. RED wall: wall-kick (jump toward it, press Space again).",
            "4. PURPLE ledge: reach via wall-kick.",
            "Watch HUD for mode changes (Mantling, etc).",
        ],
    );

    // The course runs along the -Z axis, starting from spawn.

    // -- Section 1: Stairs (auto-step) --------------------------------------
    spawn_stairs(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 0.0, 4.0),
        5,
        1.2,
        0.25,
        3.0,
    );

    // -- Section 2: Mantle block (waist-high, reachable from ground) --------
    // Height ~1.2m — the controller is ~1.8m tall, so this is waist-high.
    // Walk into it and press E to mantle on top.
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Mantle Block (press E)",
        Vec3::new(0.0, 0.6, -4.0),
        Vec3::new(3.0, 1.2, 2.0),
        Color::srgb(0.35, 0.58, 0.44),
    );

    // -- Section 3: Wall-kick wall ------------------------------------------
    // A tall wall. Jump toward it and press Space while touching it to kick off.
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Wall-Kick Wall (jump + Space)",
        Vec3::new(0.0, 2.5, -10.0),
        Vec3::new(6.0, 5.0, 0.8),
        Color::srgb(0.62, 0.32, 0.28),
    );

    // -- Section 4: Elevated ledge (reach via wall-kick) --------------------
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Elevated Ledge",
        Vec3::new(6.0, 2.5, -10.0),
        Vec3::new(4.0, 0.3, 4.0),
        Color::srgb(0.44, 0.49, 0.68),
    );

    // -- Player character (tuned for parkour) -------------------------------
    let player_transform = Transform::from_xyz(0.0, 3.0, 10.0);

    commands.spawn((
        Name::new("Player"),
        DemoPlayer,
        CharacterController {
            speed: 11.0,
            jump_input_buffer: Duration::from_millis(200),
            jump_height: 2.1,
            step_size: 0.8,
            coyote_time: Duration::from_millis(140),
            ..default()
        },
        CharacterFlying::default(),
        CharacterPush::default(),
        CharacterMantle::default(),
        CharacterWallKick::default(),
        Visibility::Inherited,
        player_transform,
        default_character_actions(),
    ));

    spawn_fps_camera(&mut commands, &player_transform);
}
