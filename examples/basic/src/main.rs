//! # Basic Character Controller
//!
//! The simplest possible working character controller: a capsule that walks, sprints,
//! crouches, and jumps on a flat ground plane with a few static obstacles.
//!
//! **Demonstrates**: plugin setup, character entity spawn, input wiring, first-person camera.

use std::time::Duration;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use saddle_character_controller::{
    AscendAction, CharacterController, CharacterControllerPlugin, CharacterControllerSystems,
    CharacterFlying, CharacterLook, CharacterPush, CrouchAction, JumpAction, LookAction,
    MoveAction, SprintAction, TraverseAction,
};
use saddle_character_controller_example_common as common;
use common::{
    DemoFixedSystems, DemoPlayer, FirstPersonCamera, animate_platforms, follow_first_person_camera,
    spawn_block, spawn_flat_ground, spawn_lighting,
};

fn main() -> AppExit {
    let mut app = common::base_app("character_controller basic");

    // -- Plugin registration ------------------------------------------------
    // `always_on(FixedUpdate)` means the controller activates at startup and runs every
    // fixed tick. In a real game you would wire activate/deactivate to your game-state
    // schedule (e.g. OnEnter(Screen::Gameplay) / OnExit(Screen::Gameplay)).
    app.add_plugins(CharacterControllerPlugin::always_on(FixedUpdate));

    // -- Systems ------------------------------------------------------------
    app.configure_sets(
        FixedUpdate,
        DemoFixedSystems::AnimatePlatforms.before(CharacterControllerSystems::Grounding),
    )
    .add_systems(Startup, setup_scene)
    .add_systems(
        FixedUpdate,
        animate_platforms.in_set(DemoFixedSystems::AnimatePlatforms),
    )
    .add_systems(PostUpdate, follow_first_person_camera);

    app.run()
}

/// Spawn lighting, ground, obstacles, the player character, and a first-person camera.
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_lighting(&mut commands);
    spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 90.0);

    // A few static crates to jump on and walk around.
    spawn_block(&mut commands, &mut meshes, &mut materials,
        "Crate Cluster A", Vec3::new(-4.0, 1.0, 2.0), Vec3::new(2.0, 2.0, 2.0),
        Color::srgb(0.56, 0.39, 0.26));
    spawn_block(&mut commands, &mut meshes, &mut materials,
        "Crate Cluster B", Vec3::new(4.5, 0.5, -3.0), Vec3::new(3.0, 1.0, 3.0),
        Color::srgb(0.39, 0.47, 0.62));
    spawn_block(&mut commands, &mut meshes, &mut materials,
        "Crate Cluster C", Vec3::new(0.0, 1.5, -10.0), Vec3::new(2.5, 3.0, 2.5),
        Color::srgb(0.31, 0.58, 0.47));

    // -- Player character ---------------------------------------------------
    // The CharacterController component holds all movement tuning. Sensible defaults are
    // provided; here we override jump buffering and coyote time for a responsive feel.
    let controller = CharacterController {
        speed: 11.0,
        jump_input_buffer: Duration::from_millis(160),
        coyote_time: Duration::from_millis(110),
        ..default()
    };

    // CharacterLook converts raw mouse/stick deltas into yaw/pitch on the controller.
    let look = CharacterLook {
        sensitivity: Vec2::splat(0.0022),
        ..default()
    };

    // Input bindings are declared inline using the `actions!` macro from bevy_enhanced_input.
    // Each action maps to a saddle-character-controller action type.
    let player = commands.spawn((
        Name::new("Player"),
        DemoPlayer,
        controller,
        look,
        CharacterFlying::default(),  // flight toggle (F by default, off at start)
        CharacterPush::default(),    // push dynamic bodies on contact
        Visibility::Inherited,
        Transform::from_xyz(0.0, 3.0, 14.0),
        actions!(CharacterController[
            (
                Action::<MoveAction>::new(),
                DeadZone::default(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
            ),
            (
                Action::<LookAction>::new(),
                Bindings::spawn((
                    Spawn((Binding::mouse_motion(), Scale::splat(0.0025))),
                    Axial::right_stick().with((Scale::splat(0.06), DeadZone::default())),
                )),
            ),
            (Action::<JumpAction>::new(), bindings![KeyCode::Space, GamepadButton::South]),
            (Action::<SprintAction>::new(), bindings![KeyCode::ShiftLeft, GamepadButton::LeftTrigger2]),
            (Action::<CrouchAction>::new(), bindings![KeyCode::ControlLeft, KeyCode::KeyC, GamepadButton::East]),
            (Action::<AscendAction>::new(), bindings![KeyCode::KeyQ, GamepadButton::LeftTrigger]),
            (Action::<TraverseAction>::new(), bindings![KeyCode::KeyE, GamepadButton::RightTrigger]),
        ]),
    )).id();

    // -- First-person camera ------------------------------------------------
    commands.spawn((
        Name::new("First Person Camera"),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::TAU / 5.5,
            ..default()
        }),
        FirstPersonCamera { target: player },
    ));
}
