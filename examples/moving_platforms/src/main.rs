//! # Moving Platforms
//!
//! Shows the character controller riding kinematic platforms that translate and rotate.
//! The controller inherits platform velocity via `SupportVelocityPolicy`, and a conveyor
//! belt shows `MovementSurface::conveyor_velocity`.
//!
//! **Demonstrates**: `SupportVelocityPolicy`, `support_detach_grace`, `MovementSurface`,
//! kinematic platform interaction, conveyor belts.

use std::time::Duration;

use bevy::prelude::*;
use common::{
    DemoFixedSystems, DemoPlayer, FirstPersonCamera, MovingPlatform, add_demo_controller_plugins,
    animate_platforms, default_character_actions, follow_first_person_camera,
    spawn_demo_instructions, spawn_flat_ground, spawn_lighting, spawn_platform,
};
use saddle_character_controller::{
    CharacterController, CharacterControllerSystems, CharacterFlying, CharacterLook, CharacterPush,
    MovementSurface, SupportVelocityPolicy,
};
use saddle_character_controller_example_common as common;

fn main() -> AppExit {
    let mut app = common::base_app("character_controller moving_platforms");
    add_demo_controller_plugins(&mut app);

    app.configure_sets(
        FixedUpdate,
        // Platforms must move before the controller resolves grounding, so the character
        // sees the new platform position/velocity on the same tick.
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
    spawn_demo_instructions(
        &mut commands,
        "Moving Platforms",
        &[
            "Ride the moving platforms and conveyor, then inspect support inheritance values in the pane.",
        ],
    );

    // -- Platform A: slides left-right along the X axis ---------------------
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Platform A",
        Vec3::new(-8.0, 1.0, 0.0),
        Vec3::new(3.5, 0.35, 3.5),
        Color::srgb(0.82, 0.57, 0.2),
        Some(MovingPlatform {
            origin: Vec3::new(-8.0, 1.0, 0.0),
            translation_axis: Vec3::X,
            translation_amplitude: 5.0,
            translation_speed: 0.9,
            rotation_speed: 0.0,
            phase: 0.0,
        }),
        None,
    );

    // -- Platform B: slides along Z with full velocity inheritance ----------
    // The `MovementSurface` override on this platform forces `SupportVelocityPolicy::Full`
    // regardless of the controller's own setting. This means the character inherits both
    // horizontal and vertical platform velocity.
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Platform B",
        Vec3::new(4.0, 2.2, -3.5),
        Vec3::new(3.0, 0.35, 3.0),
        Color::srgb(0.31, 0.52, 0.84),
        Some(MovingPlatform {
            origin: Vec3::new(4.0, 2.2, -3.5),
            translation_axis: Vec3::Z,
            translation_amplitude: 4.0,
            translation_speed: 1.1,
            rotation_speed: 0.0,
            phase: 1.3,
        }),
        Some(MovementSurface {
            inherit_velocity_policy: Some(SupportVelocityPolicy::Full),
            ..default()
        }),
    );

    // -- Conveyor belt: static platform with surface conveyor velocity ------
    // The character slides sideways while standing on this surface even without input.
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Conveyor Strip",
        Vec3::new(16.0, 0.4, 0.0),
        Vec3::new(8.0, 0.2, 3.0),
        Color::srgb(0.42, 0.37, 0.76),
        None, // not moving — it is a static conveyor
        Some(MovementSurface {
            conveyor_velocity: Vec3::new(2.5, 0.0, 0.0),
            inherit_velocity_policy: Some(SupportVelocityPolicy::Horizontal),
            ..default()
        }),
    );

    // -- Player character ---------------------------------------------------
    // Key override: `support_velocity_policy = Full` means the controller inherits
    // platform movement by default. `support_detach_grace` gives a short window after
    // leaving a platform where the inherited velocity is preserved.
    let controller = CharacterController {
        speed: 11.0,
        jump_input_buffer: Duration::from_millis(160),
        coyote_time: Duration::from_millis(110),
        support_velocity_policy: SupportVelocityPolicy::Full,
        support_detach_grace: Duration::from_millis(180),
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
            Visibility::Inherited,
            Transform::from_xyz(-16.0, 3.0, 0.0),
            default_character_actions(),
        ))
        .id();

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
