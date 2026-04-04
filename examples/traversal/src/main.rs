//! # Traversal
//!
//! An obstacle course for parkour-style movement: mantling up ledges, wall-kicking
//! off vertical surfaces, climbing stairs, and landing on elevated platforms.
//!
//! **Demonstrates**: `CharacterMantle`, `CharacterWallKick`, `step_size`, `jump_height`,
//! `coyote_time`, debug draw overlay.

use std::time::Duration;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use saddle_character_controller::{
    AscendAction, CharacterController, CharacterControllerDebugDraw,
    CharacterControllerPlugin, CharacterControllerSystems, CharacterFlying, CharacterLook,
    CharacterMantle, CharacterPush, CharacterWallKick, CrouchAction, JumpAction, LookAction,
    MoveAction, SprintAction, TraverseAction,
};
use saddle_character_controller_example_common as common;
use common::{
    DemoFixedSystems, DemoPlayer, FirstPersonCamera, animate_platforms, follow_first_person_camera,
    spawn_block, spawn_flat_ground, spawn_lighting, spawn_platform, spawn_stairs,
};

fn main() -> AppExit {
    let mut app = common::base_app("character_controller traversal");
    app.add_plugins(CharacterControllerPlugin::always_on(FixedUpdate));

    // Debug draw helps visualize mantle detection and wall-kick traces.
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
    )
    .add_systems(PostUpdate, follow_first_person_camera);

    app.run()
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_lighting(&mut commands);
    spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 120.0);

    // -- Mantle block: a waist-high obstacle the player can vault over ------
    // Press the traverse key (E) while moving into the block and looking at it.
    spawn_block(
        &mut commands, &mut meshes, &mut materials,
        "Mantle Block",
        Vec3::new(0.0, 0.75, -2.0),
        Vec3::new(2.0, 1.5, 2.0),
        Color::srgb(0.35, 0.58, 0.44),
    );

    // -- Tall wall: too high to mantle, but can wall-kick off ---------------
    // Jump toward it and press jump again while touching the wall to kick off.
    spawn_block(
        &mut commands, &mut meshes, &mut materials,
        "Tall Wall",
        Vec3::new(8.0, 2.0, -6.0),
        Vec3::new(1.0, 4.0, 8.0),
        Color::srgb(0.62, 0.32, 0.28),
    );

    // -- Staircase ----------------------------------------------------------
    spawn_stairs(
        &mut commands, &mut meshes, &mut materials,
        Vec3::new(-10.0, 0.0, -8.0),
        6, 1.0, 0.3, 2.0,
    );

    // -- Elevated ledge to reach via wall-kick or mantle combo --------------
    spawn_platform(
        &mut commands, &mut meshes, &mut materials,
        "Traversal Ledge",
        Vec3::new(14.0, 2.6, 4.0),
        Vec3::new(6.0, 0.3, 3.0),
        Color::srgb(0.44, 0.49, 0.68),
        None,
        None,
    );

    // -- Player character (tuned for parkour) -------------------------------
    let controller = CharacterController {
        speed: 11.0,
        jump_input_buffer: Duration::from_millis(160),
        // Higher jump for reaching ledges
        jump_height: 2.1,
        // Larger step-up for clambering small obstacles
        step_size: 0.8,
        // Extended coyote time for forgiving ledge jumps
        coyote_time: Duration::from_millis(140),
        ..default()
    };
    let look = CharacterLook {
        sensitivity: Vec2::splat(0.0022),
        ..default()
    };

    let player = commands.spawn((
        Name::new("Player"),
        DemoPlayer,
        controller,
        look,
        CharacterFlying::default(),
        CharacterPush::default(),
        CharacterMantle::default(),   // enable ledge mantling
        CharacterWallKick::default(), // enable wall-kick
        Visibility::Inherited,
        Transform::from_xyz(-12.0, 3.0, 12.0),
        actions!(CharacterController[
            (Action::<MoveAction>::new(), DeadZone::default(), Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick()))),
            (Action::<LookAction>::new(), Bindings::spawn((Spawn((Binding::mouse_motion(), Scale::splat(0.0025))), Axial::right_stick().with((Scale::splat(0.06), DeadZone::default()))))),
            (Action::<JumpAction>::new(), bindings![KeyCode::Space, GamepadButton::South]),
            (Action::<SprintAction>::new(), bindings![KeyCode::ShiftLeft, GamepadButton::LeftTrigger2]),
            (Action::<CrouchAction>::new(), bindings![KeyCode::ControlLeft, KeyCode::KeyC, GamepadButton::East]),
            (Action::<AscendAction>::new(), bindings![KeyCode::KeyQ, GamepadButton::LeftTrigger]),
            (Action::<TraverseAction>::new(), bindings![KeyCode::KeyE, GamepadButton::RightTrigger]),
        ]),
    )).id();

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
