//! # Third Person Camera
//!
//! The character controller with a trailing third-person camera instead of first-person.
//! The character body is rendered as a visible cube. Includes jump platforms to verify
//! jumping works in third-person.
//!
//! **Demonstrates**: third-person camera integration, visible player body, mouse look.
//! **Test**: walk, jump on platforms, sprint. Verify jump works on flat ground.

use std::time::Duration;

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};
use common::{
    CursorLockState, DemoFixedSystems, DemoPlayer, ThirdPersonCamera, add_demo_controller_plugins,
    add_diagnostic_hud, animate_platforms, default_character_actions, follow_third_person_camera,
    spawn_block, spawn_controller_visual, spawn_demo_instructions, spawn_flat_ground,
    spawn_lighting, spawn_stairs,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerState, CharacterControllerSystems, CharacterFlying,
    CharacterPush,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller third_person");
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
    )
    .add_systems(Update, third_person_mouse_look)
    .add_systems(PostUpdate, follow_third_person_camera);

    app.run()
}

fn third_person_mouse_look(
    lock_state: Res<CursorLockState>,
    mouse: Res<AccumulatedMouseMotion>,
    mut players: Query<&mut CharacterControllerState, With<DemoPlayer>>,
) {
    if !lock_state.0 {
        return;
    }
    let Ok(mut state) = players.single_mut() else {
        return;
    };
    let sensitivity = 0.0022;
    let (yaw, pitch, _) = state.orientation.to_euler(EulerRot::YXZ);
    let new_yaw = yaw - mouse.delta.x * sensitivity;
    let new_pitch = (pitch - mouse.delta.y * sensitivity).clamp(-1.5, 1.5);
    state.orientation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, 0.0);
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
            "Walk and jump on the flat ground.",
            "Jump onto the colored platforms.",
            "Climb the stairs.",
        ],
    );

    // -- Jump platforms (same as basic) -------------------------------------
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Low Platform",
        Vec3::new(4.0, 0.4, 0.0),
        Vec3::new(3.0, 0.8, 3.0),
        Color::srgb(0.35, 0.58, 0.44),
    );
    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Medium Platform",
        Vec3::new(4.0, 0.9, -5.0),
        Vec3::new(3.0, 1.8, 3.0),
        Color::srgb(0.56, 0.49, 0.26),
    );

    // -- Stairs -------------------------------------------------------------
    spawn_stairs(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-8.0, 0.0, -4.0),
        6,
        1.0,
        0.3,
        2.0,
    );

    // -- Player character ---------------------------------------------------
    let controller = CharacterController {
        speed: 10.0,
        sprint_speed_scale: 1.35,
        jump_input_buffer: Duration::from_millis(160),
        coyote_time: Duration::from_millis(110),
        ..default()
    };
    let player = commands
        .spawn((
            Name::new("Player"),
            DemoPlayer,
            controller,
            CharacterFlying::default(),
            CharacterPush::default(),
            Visibility::Inherited,
            Transform::from_xyz(0.0, 3.0, 12.0),
            default_character_actions(),
        ))
        .id();

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
