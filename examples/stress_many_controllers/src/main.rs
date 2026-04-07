//! # Stress Test — Many Controllers
//!
//! Spawns 50 character controllers in a grid to stress-test physics performance.
//! An overhead camera watches them all. Only the first controller (the "player") has
//! input bindings; the rest are idle NPCs that share the same physics pipeline.
//!
//! **Demonstrates**: scaling behavior, many simultaneous controllers.
//! **Test**: stable frame rate, no crashes with 50 controllers.

use bevy::prelude::*;
use common::{
    DemoPlayer, add_demo_controller_plugins, add_diagnostic_hud, default_character_actions,
    spawn_block, spawn_demo_instructions, spawn_flat_ground, spawn_lighting,
};
use saddle_character_controller::{CharacterController, CharacterFlying, CharacterPush};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller stress_test");

    add_demo_controller_plugins(&mut app);
    add_diagnostic_hud(&mut app);

    app.add_systems(Startup, setup_scene);

    app.run()
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_lighting(&mut commands);
    spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 200.0);
    spawn_demo_instructions(
        &mut commands,
        "Stress Test (50 controllers)",
        &["ORANGE = player (you). GREY = idle NPCs. Watch frame rate."],
    );

    // -- Pillars for collision variety --------------------------------------
    for x in [-10.0_f32, 0.0, 10.0] {
        for z in [-10.0_f32, 0.0, 10.0] {
            spawn_block(
                &mut commands,
                &mut meshes,
                &mut materials,
                "Pillar",
                Vec3::new(x, 1.5, z),
                Vec3::new(1.2, 3.0, 1.2),
                Color::srgb(0.4, 0.4, 0.45),
            );
        }
    }

    // -- Player controller (orange, has input) ------------------------------
    commands.spawn((
        Name::new("Player"),
        DemoPlayer,
        CharacterController::default(),
        CharacterFlying::default(),
        CharacterPush::default(),
        Visibility::Inherited,
        Transform::from_xyz(0.0, 3.0, 15.0),
        default_character_actions(),
    ));

    // -- 49 bot controllers (grey, no input) --------------------------------
    for i in 0..49 {
        let row = i / 7;
        let col = i % 7;
        let x = (col as f32 - 3.0) * 4.0;
        let z = -(row as f32) * 4.0 - 5.0;
        commands.spawn((
            Name::new(format!("Bot {i}")),
            CharacterController::default(),
            Visibility::Inherited,
            Transform::from_xyz(x, 3.0, z),
        ));
    }

    // -- Overhead camera ----------------------------------------------------
    commands.spawn((
        Name::new("Overhead Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 35.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
