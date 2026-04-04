//! # Slopes and Stairs
//!
//! Shows how the character controller handles angled surfaces and stepped geometry.
//! Includes a walkable ramp, a steep slide-only ramp (`MovementSurface::slide_only`),
//! and a staircase that the controller auto-steps.
//!
//! **Demonstrates**: `MovementSurface`, `step_size`, ramp traction, slope sliding.

use std::time::Duration;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use saddle_character_controller::{
    AscendAction, CharacterController, CharacterControllerPlugin, CharacterControllerSystems,
    CharacterFlying, CharacterLook, CharacterPush, CrouchAction, JumpAction, LookAction,
    MoveAction, MovementSurface, SprintAction, TraverseAction,
};
use saddle_character_controller_example_common as common;
use common::{
    DemoFixedSystems, DemoPlayer, FirstPersonCamera, animate_platforms, follow_first_person_camera,
    spawn_flat_ground, spawn_lighting, spawn_ramp, spawn_stairs,
};

fn main() -> AppExit {
    let mut app = common::base_app("character_controller slopes_and_stairs");
    app.add_plugins(CharacterControllerPlugin::always_on(FixedUpdate));

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

    // -- Slopes -------------------------------------------------------------
    // A gentle ramp the player can walk up normally.
    spawn_ramp(
        &mut commands, &mut meshes, &mut materials,
        "Walkable Ramp",
        Vec3::new(-6.0, 1.2, 0.0),
        Vec3::new(8.0, 0.8, 8.0),
        -0.4, // tilt angle (radians around Z)
        Color::srgb(0.23, 0.49, 0.41),
        None, // default surface — walkable
    );

    // A steep ramp marked as slide-only. The controller cannot gain traction here and
    // will slide back down. This is configured via `MovementSurface::slide_only`.
    spawn_ramp(
        &mut commands, &mut meshes, &mut materials,
        "Steep Ramp",
        Vec3::new(9.0, 1.2, 0.0),
        Vec3::new(8.0, 0.8, 8.0),
        -1.0,
        Color::srgb(0.63, 0.28, 0.24),
        Some(MovementSurface {
            slide_only: true,
            ..default()
        }),
    );

    // -- Stairs -------------------------------------------------------------
    // A 7-step staircase. The controller's `step_size` determines the maximum step height
    // it can auto-climb without jumping.
    spawn_stairs(
        &mut commands, &mut meshes, &mut materials,
        Vec3::new(-18.0, 0.0, -5.0),
        7,    // number of steps
        1.2,  // step depth
        0.25, // step height
        2.0,  // step width
    );

    // -- Player (default controller, no special overrides) ------------------
    let controller = CharacterController {
        speed: 11.0,
        jump_input_buffer: Duration::from_millis(160),
        coyote_time: Duration::from_millis(110),
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
        Visibility::Inherited,
        Transform::from_xyz(-14.0, 3.0, 12.0),
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
