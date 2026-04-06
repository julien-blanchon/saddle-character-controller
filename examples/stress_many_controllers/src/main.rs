//! # Stress Test — Many Controllers
//!
//! Spawns 50 character controllers in a grid to stress-test physics performance.
//! An overhead camera watches them all. Only the first controller (the "player") has
//! input bindings; the rest are idle NPCs that share the same physics pipeline.
//!
//! **Demonstrates**: scaling behavior, many simultaneous controllers, overview camera,
//! per-entity `CharacterPush` toggling.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, add_demo_controller_plugins, animate_platforms,
    default_character_actions, spawn_block, spawn_controller_visual, spawn_demo_instructions,
    spawn_flat_ground, spawn_lighting,
};
use saddle_character_controller::{CharacterController, CharacterControllerSystems, CharacterLook};
use saddle_character_controller_example_common as common;

const BOT_COUNT: usize = 49; // plus 1 player = 50 total

fn main() -> AppExit {
    let mut app = common::base_app("character_controller stress_many_controllers");
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
    spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 180.0);
    spawn_demo_instructions(
        &mut commands,
        "Stress Test",
        &[
            "Only the orange player accepts input. The remaining controllers are passive runtime load for perf inspection.",
        ],
    );

    // A row of pillars for visual reference / collision variety.
    for i in 0..8 {
        let x = -30.0 + i as f32 * 8.0;
        spawn_block(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Stress Pillar {i}"),
            Vec3::new(x, 1.5, -14.0),
            Vec3::new(2.0, 3.0, 2.0),
            Color::srgb(0.4, 0.42, 0.48),
        );
    }

    // -- Player (the only one with input) -----------------------------------
    let controller = CharacterController {
        speed: 9.0,
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
            // No CharacterPush — disabled in stress test to reduce pair-wise interactions.
            Visibility::Inherited,
            Transform::from_xyz(0.0, 3.0, 16.0),
            default_character_actions(),
        ))
        .id();

    // Visible body for the player (overview cam needs to see it).
    spawn_controller_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        player,
        Color::srgb(0.92, 0.44, 0.22),
    );

    // -- Bot controllers (no input, just physics) ---------------------------
    let columns = 10usize;
    let spacing = 4.5;
    let mut spawned = 0usize;

    for row in 0.. {
        for column in 0..columns {
            if spawned >= BOT_COUNT {
                break;
            }
            let x = column as f32 * spacing - 20.0;
            let z = row as f32 * spacing - 12.0;
            let bot = commands
                .spawn((
                    Name::new(format!("Bot {spawned:02}")),
                    CharacterController {
                        speed: 8.0,
                        jump_height: 1.4,
                        max_speed: 40.0,
                        ..default()
                    },
                    Visibility::Inherited,
                    Transform::from_xyz(x, 3.0, z),
                ))
                .id();
            let tint = Color::srgb(0.25 + 0.02 * column as f32, 0.35 + 0.01 * row as f32, 0.75);
            spawn_controller_visual(&mut commands, &mut meshes, &mut materials, bot, tint);
            spawned += 1;
        }
        if spawned >= BOT_COUNT {
            break;
        }
    }

    // -- Overview camera (static, looking down at the grid) -----------------
    commands.spawn((
        Name::new("Overview Camera"),
        Camera3d::default(),
        Transform::from_xyz(22.0, 28.0, 26.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
