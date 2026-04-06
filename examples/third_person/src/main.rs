//! # Third Person Camera
//!
//! The character controller with a trailing third-person camera instead of first-person.
//! The character body is rendered as a visible cube, and the camera follows behind at a
//! configurable distance and height. Includes obstacles plus mantling for variety.
//!
//! **Demonstrates**: third-person camera integration, visible player body, `CharacterMantle`,
//! `CharacterSwimming`, `sprint_speed_scale`.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, ThirdPersonCamera, add_demo_controller_plugins,
    animate_platforms, default_character_actions, follow_third_person_camera, spawn_block,
    spawn_controller_visual, spawn_demo_instructions, spawn_flat_ground, spawn_lighting,
    spawn_platform, spawn_stairs,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerSystems, CharacterFlying, CharacterLook,
    CharacterMantle, CharacterPush, convenience::environment::CharacterSwimming,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller third_person");
    add_demo_controller_plugins(&mut app);

    app.configure_sets(
        FixedUpdate,
        DemoFixedSystems::AnimatePlatforms.before(CharacterControllerSystems::Grounding),
    )
    .add_systems(Startup, setup_scene)
    .add_systems(
        FixedUpdate,
        animate_platforms.in_set(DemoFixedSystems::AnimatePlatforms),
    )
    .add_systems(PostUpdate, follow_third_person_camera);

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
        "Third Person",
        &[
            "Watch the visible body and camera framing while running the course. Mantle, sprint, and swimming are enabled.",
        ],
    );

    // -- Obstacles (same as basic + traversal) ------------------------------
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

    // Traversal geometry — mantle blocks, tall wall, stairs, ledge.
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Mantle Block",
        Vec3::new(0.0, 0.75, -2.0),
        Vec3::new(2.0, 1.5, 2.0),
        Color::srgb(0.35, 0.58, 0.44),
    );
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Tall Wall",
        Vec3::new(8.0, 2.0, -6.0),
        Vec3::new(1.0, 4.0, 8.0),
        Color::srgb(0.62, 0.32, 0.28),
    );
    spawn_stairs(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-10.0, 0.0, -8.0),
        6,
        1.0,
        0.3,
        2.0,
    );
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Traversal Ledge",
        Vec3::new(14.0, 2.6, 4.0),
        Vec3::new(6.0, 0.3, 3.0),
        Color::srgb(0.44, 0.49, 0.68),
        None,
        None,
    );

    // -- Player character (third-person tuning) -----------------------------
    let controller = CharacterController {
        speed: 10.0,
        sprint_speed_scale: 1.35,
        jump_input_buffer: Duration::from_millis(160),
        coyote_time: Duration::from_millis(110),
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
            CharacterSwimming::default(),
            CharacterMantle::default(),
            Visibility::Inherited,
            Transform::from_xyz(0.0, 3.0, 12.0),
            default_character_actions(),
        ))
        .id();

    // The player needs a visible body in third-person (in first-person it is invisible).
    spawn_controller_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        player,
        Color::srgb(0.92, 0.44, 0.22),
    );

    // -- Third-person camera ------------------------------------------------
    commands.spawn((
        Name::new("Third Person Camera"),
        Camera3d::default(),
        ThirdPersonCamera {
            target: player,
            distance: 6.5,
            height: 2.8,
        },
    ));
}
