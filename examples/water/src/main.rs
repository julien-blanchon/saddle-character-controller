//! # Water / Swimming
//!
//! Shows the character controller transitioning between walking and swimming when
//! entering a `SwimVolume` trigger. The controller automatically switches to swim
//! mode: buoyancy, reduced gravity, and different movement feel.
//!
//! **Demonstrates**: `CharacterSwimming`, `SwimVolume`, water-to-ground transitions,
//! `AscendAction` for swimming upward.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, add_demo_controller_plugins, animate_platforms,
    default_character_actions, spawn_block, spawn_demo_instructions, spawn_flat_ground,
    spawn_fps_camera, spawn_lighting, spawn_water_volume,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerSystems, CharacterFlying, CharacterPush,
    CharacterSwimming,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller water");
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
    spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 120.0);
    spawn_demo_instructions(
        &mut commands,
        "Swimming",
        &[
            "Drop into the pool, then hold Q to rise while submerged. Compare walk and swim tuning in the pane.",
        ],
    );

    // Some crates to jump on near the pool edge.
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

    // -- Pool ---------------------------------------------------------------
    // A raised rim around the pool so the character must jump in or walk off the edge.
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Pool Rim",
        Vec3::new(6.0, 0.2, 0.0),
        Vec3::new(18.0, 0.4, 14.0),
        Color::srgb(0.36, 0.34, 0.31),
    );

    // The swim volume is a sensor collider with the `SwimVolume` helper component.
    // When the character's feet enter this volume, `CharacterSwimming` activates.
    spawn_water_volume(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Pool",
        Vec3::new(6.0, 1.4, 0.0),
        Vec3::new(12.0, 3.0, 8.0),
    );

    // -- Player character with swimming enabled -----------------------------
    let controller = CharacterController {
        speed: 11.0,
        jump_input_buffer: Duration::from_millis(160),
        coyote_time: Duration::from_millis(110),
        ..default()
    };
    let player_transform = Transform::from_xyz(-14.0, 3.0, 10.0);

    commands.spawn((
        Name::new("Player"),
        DemoPlayer,
        controller,
        CharacterFlying::default(),
        CharacterPush::default(),
        // Swimming must be added as a component for the controller to enter swim mode.
        // Without this, the character would fall through water volumes normally.
        CharacterSwimming::default(),
        Visibility::Inherited,
        player_transform,
        default_character_actions(),
    ));

    spawn_fps_camera(&mut commands, &player_transform);
}
