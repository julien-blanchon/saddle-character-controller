//! # Slopes and Stairs
//!
//! Shows how the character controller handles angled surfaces and stepped geometry.
//! Includes a walkable ramp, a steep slide-only ramp, and a staircase that the controller
//! auto-steps.
//!
//! **Demonstrates**: `MovementSurface::slide_only`, `step_size`, `min_walk_angle`.
//! **Test**: walk up gentle ramp, slide on steep ramp, auto-climb stairs.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, add_demo_controller_plugins, add_diagnostic_hud,
    animate_platforms, default_character_actions, spawn_block, spawn_demo_instructions,
    spawn_flat_ground, spawn_fps_camera, spawn_lighting, spawn_ramp, spawn_stairs,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerSystems, CharacterFlying, CharacterPush,
    MovementSurface,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller slopes_and_stairs");

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
        "Slopes & Stairs",
        &[
            "GREEN ramp: walkable (gentle angle).",
            "RED ramp: slide-only (too steep).",
            "BLUE stairs: auto-step climbing.",
            "Watch the HUD for mode changes.",
        ],
    );

    // -- Gentle ramp (walkable) — about 22 degrees --------------------------
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Walkable Ramp",
        Vec3::new(-6.0, 1.2, -4.0),
        Vec3::new(8.0, 0.8, 6.0),
        -0.4,
        Color::srgb(0.23, 0.49, 0.41),
        None,
    );

    // -- Steep ramp (slide-only) — about 57 degrees -------------------------
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Steep Ramp (Slide Only)",
        Vec3::new(8.0, 1.2, -4.0),
        Vec3::new(8.0, 0.8, 6.0),
        -1.0,
        Color::srgb(0.63, 0.28, 0.24),
        Some(MovementSurface {
            slide_only: true,
            ..default()
        }),
    );

    // -- Staircase (7 steps) ------------------------------------------------
    spawn_stairs(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 0.0, -12.0),
        7,
        1.0,
        0.25,
        3.0,
    );

    // Landing platform at top of stairs
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Stair Landing",
        Vec3::new(0.0, 0.875, -19.75),
        Vec3::new(3.0, 1.75, 1.5),
        Color::srgb(0.34, 0.41, 0.52),
    );

    // -- Player character ---------------------------------------------------
    let player_transform = Transform::from_xyz(0.0, 3.0, 8.0);

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
