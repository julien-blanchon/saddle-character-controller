#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use std::{fmt::Write as _, time::Duration};

use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_enhanced_input::prelude::*;
use saddle_animation_ik::{
    IkChain, IkChainState, IkConstraint, IkDebugSettings, IkJoint, IkPlugin, IkSystems,
    LookAtTarget,
};
use saddle_character_controller::{
    AccumulatedInput, CharacterController, CharacterControllerPlugin, CharacterControllerState,
    CharacterControllerSystems, CharacterFlying, CharacterLook, CharacterMantle,
    CharacterMotionStats, CharacterPush, CharacterWallKick, FlightCollisionMode, MovementSurface,
    SupportRotationPolicy, SupportVelocityPolicy,
    adapters::enhanced_input::{
        AscendAction, CharacterControllerEnhancedInputPlugin, CrouchAction, JumpAction, LookAction,
        MoveAction, SprintAction, TraverseAction,
    },
    convenience::environment::{
        CharacterControllerEnvironmentPlugin, CharacterSwimming, SwimVolume,
    },
};
use saddle_character_state_machine::{
    CharacterAnimationFacts, CharacterAnimationSelection, CharacterStateMachine,
    CharacterStateMachineDefinition, CharacterStateMachineLibrary, CharacterStateMachinePlugin,
    CharacterStateMachineRuntime, CharacterStateMachineSystems, LocomotionMode, StateDefinition,
    TransitionCondition, TransitionDefinition,
};
use saddle_pane::prelude::*;

#[derive(Component)]
struct LabController;

#[derive(Component)]
struct LabCamera {
    distance: f32,
    height: f32,
}

#[derive(Component)]
struct LabOverlay;

#[derive(Component)]
struct LabBody;

#[derive(Component)]
struct LabLookController;

#[derive(Component)]
struct LabLookTarget;

#[derive(Resource, Default, Clone, Debug, Reflect)]
#[reflect(Resource)]
struct LabDiagnostics {
    controller_position: Vec3,
    movement_mode: String,
    grounded_on: String,
    support_angular_speed: f32,
    current_speed: f32,
    animation_state: String,
    animation_binding: String,
    look_error: f32,
}

#[derive(Component)]
struct MovingPlatform {
    origin: Vec3,
    translation_axis: Vec3,
    translation_amplitude: f32,
    translation_speed: f32,
    rotation_speed: f32,
    phase: f32,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LabFixedSystems {
    AnimatePlatforms,
}

#[derive(Resource, Pane)]
#[pane(title = "Character Controller Lab")]
struct LabPane {
    #[pane(tab = "Movement", slider, min = 4.0, max = 20.0, step = 0.1)]
    speed: f32,
    #[pane(tab = "Movement", slider, min = 1.0, max = 2.5, step = 0.05)]
    sprint_speed_scale: f32,
    #[pane(tab = "Movement", slider, min = 0.5, max = 3.0, step = 0.05)]
    jump_height: f32,
    #[pane(tab = "Movement", slider, min = 0.1, max = 1.2, step = 0.05)]
    step_size: f32,
    #[pane(tab = "Movement", slider, min = 0.0005, max = 0.01, step = 0.0001)]
    look_sensitivity: f32,
    #[pane(tab = "Movement")]
    inherit_support_yaw: bool,
    #[pane(tab = "Traversal")]
    flight_enabled: bool,
    #[pane(tab = "Traversal", slider, min = 4.0, max = 30.0, step = 0.5)]
    flight_speed: f32,
    #[pane(tab = "Traversal")]
    flight_noclip: bool,
    #[pane(tab = "IK", slider, min = 1.5, max = 8.0, step = 0.1)]
    look_distance: f32,
    #[pane(tab = "IK", slider, min = 0.0, max = 1.0, step = 0.05)]
    look_weight: f32,
    #[pane(tab = "Runtime", monitor)]
    movement_mode: String,
    #[pane(tab = "Runtime", monitor)]
    grounded_on: String,
    #[pane(tab = "Runtime", monitor)]
    current_speed: f32,
    #[pane(tab = "Runtime", monitor)]
    support_angular_speed: f32,
    #[pane(tab = "Runtime", monitor)]
    animation_state: String,
    #[pane(tab = "Runtime", monitor)]
    animation_binding: String,
    #[pane(tab = "Runtime", monitor)]
    look_error: f32,
}

impl Default for LabPane {
    fn default() -> Self {
        Self {
            speed: 12.0,
            sprint_speed_scale: 1.5,
            jump_height: 1.8,
            step_size: 0.7,
            look_sensitivity: 0.0023,
            inherit_support_yaw: true,
            flight_enabled: false,
            flight_speed: 14.0,
            flight_noclip: false,
            look_distance: 4.8,
            look_weight: 1.0,
            movement_mode: "Airborne".into(),
            grounded_on: "none".into(),
            current_speed: 0.0,
            support_angular_speed: 0.0,
            animation_state: "Idle".into(),
            animation_binding: "idle".into(),
            look_error: 0.0,
        }
    }
}

fn pane_plugins() -> (
    bevy_flair::FlairPlugin,
    bevy_input_focus::InputDispatchPlugin,
    bevy_ui_widgets::UiWidgetsPlugins,
    bevy_input_focus::tab_navigation::TabNavigationPlugin,
    saddle_pane::PanePlugin,
) {
    (
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        saddle_pane::PanePlugin,
    )
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.init_resource::<LabDiagnostics>();
    app.init_resource::<LabPane>();
    app.register_type::<LabDiagnostics>();
    app.register_pane::<LabPane>();
    app.add_plugins((
        DefaultPlugins,
        PhysicsPlugins::default(),
        CharacterControllerPlugin::always_on(FixedUpdate),
        CharacterControllerEnhancedInputPlugin,
        CharacterControllerEnvironmentPlugin::new(FixedUpdate),
        CharacterStateMachinePlugin::always_on(Update),
        IkPlugin::default(),
        pane_plugins(),
    ));
    app.insert_resource(IkDebugSettings {
        enabled: false,
        ..default()
    });
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::CharacterControllerLabE2EPlugin);
    app.configure_sets(
        FixedUpdate,
        LabFixedSystems::AnimatePlatforms.before(CharacterControllerSystems::Grounding),
    );
    app.add_systems(Startup, setup);
    app.add_systems(
        FixedUpdate,
        animate_platforms.in_set(LabFixedSystems::AnimatePlatforms),
    );
    app.add_systems(
        Update,
        (
            sync_animation_facts.before(CharacterStateMachineSystems::GatherFacts),
            sync_look_target.before(IkSystems::Prepare),
            (sync_body_visuals, sync_lab_pane, sync_overlay)
                .chain()
                .after(CharacterStateMachineSystems::ApplyAnimation)
                .after(IkSystems::Apply),
            capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
            release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
        ),
    );
    app.add_systems(PostUpdate, follow_camera);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut library: ResMut<CharacterStateMachineLibrary>,
) {
    commands.spawn((
        Name::new("Lab Sun"),
        DirectionalLight {
            illuminance: 22_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(12.0, 18.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Name::new("Lab Ground"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(80.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.18, 0.17),
            perceptual_roughness: 1.0,
            ..default()
        })),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    spawn_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Mantle Block",
        Vec3::new(0.0, 0.8, -2.0),
        Vec3::new(2.2, 1.6, 2.2),
        Color::srgb(0.36, 0.57, 0.44),
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Walkable Ramp",
        Vec3::new(-8.0, 1.1, -2.5),
        Vec3::new(8.0, 0.8, 8.0),
        -0.42,
        Color::srgb(0.25, 0.50, 0.42),
        None,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Surf Ramp",
        Vec3::new(12.0, 2.0, 2.5),
        Vec3::new(12.0, 0.5, 10.0),
        -1.0,
        Color::srgb(0.72, 0.35, 0.29),
        Some(MovementSurface {
            slide_only: true,
            traction_multiplier: 0.25,
            ..default()
        }),
    );
    spawn_stairs(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-16.0, 0.0, -10.0),
        6,
        1.1,
        0.26,
        2.4,
    );
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Platform A",
        Vec3::new(-6.0, 1.2, 5.0),
        Vec3::new(3.5, 0.35, 3.5),
        Color::srgb(0.81, 0.58, 0.22),
        Some(MovingPlatform {
            origin: Vec3::new(-6.0, 1.2, 5.0),
            translation_axis: Vec3::X,
            translation_amplitude: 4.0,
            translation_speed: 0.7,
            rotation_speed: 0.25,
            phase: 0.0,
        }),
        Some(MovementSurface {
            inherit_velocity_policy: Some(SupportVelocityPolicy::Full),
            ..default()
        }),
    );
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Conveyor",
        Vec3::new(10.0, 0.35, -8.0),
        Vec3::new(8.0, 0.3, 3.0),
        Color::srgb(0.41, 0.38, 0.77),
        None,
        Some(MovementSurface {
            conveyor_velocity: Vec3::new(2.0, 0.0, 0.0),
            inherit_velocity_policy: Some(SupportVelocityPolicy::Horizontal),
            ..default()
        }),
    );
    spawn_water_volume(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Pool",
        Vec3::new(16.0, 1.6, 10.0),
        Vec3::new(10.0, 3.0, 8.0),
    );
    spawn_dynamic_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Push Crate",
        Vec3::new(2.5, 1.0, 3.0),
        Vec3::new(1.2, 1.2, 1.2),
        Color::srgb(0.56, 0.40, 0.28),
    );

    let definition_id = library.register(build_lab_animation_definition()).unwrap();
    spawn_look_target(&mut commands, &mut meshes, &mut materials);
    spawn_controller(&mut commands, &mut meshes, &mut materials, definition_id);

    commands.spawn((
        Name::new("Lab Camera"),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::TAU / 5.6,
            ..default()
        }),
        LabCamera {
            distance: 6.8,
            height: 2.8,
        },
        Transform::from_xyz(0.0, 5.5, 18.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Character Controller Overlay"),
        LabOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(14.0)),
            border_radius: BorderRadius::all(Val::Px(16.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.78)),
        Text::default(),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn spawn_controller(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    definition_id: saddle_character_state_machine::CharacterStateMachineDefinitionId,
) -> Entity {
    let player = commands
        .spawn((
            Name::new("Lab Controller"),
            LabController,
            CharacterStateMachine::new(definition_id),
            CharacterAnimationFacts {
                grounded: true,
                locomotion_mode: LocomotionMode::Idle,
                ..default()
            },
            CharacterController {
                support_velocity_policy: SupportVelocityPolicy::Full,
                support_detach_grace: Duration::from_millis(180),
                coyote_time: Duration::from_millis(130),
                jump_input_buffer: Duration::from_millis(170),
                ..default()
            },
            CharacterLook {
                sensitivity: Vec2::splat(0.0023),
                ..default()
            },
            CharacterSwimming::default(),
            CharacterMantle::default(),
            CharacterWallKick::default(),
            CharacterPush::default(),
            CharacterFlying {
                enabled: false,
                collision_mode: FlightCollisionMode::NoClip,
                ..default()
            },
            Visibility::Inherited,
            Transform::from_xyz(0.0, 2.5, 14.0),
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
                (
                    Action::<SprintAction>::new(),
                    bindings![KeyCode::ShiftLeft, GamepadButton::LeftTrigger2],
                ),
                (
                    Action::<CrouchAction>::new(),
                    bindings![KeyCode::ControlLeft, KeyCode::KeyC, GamepadButton::East],
                ),
                (
                    Action::<AscendAction>::new(),
                    bindings![KeyCode::KeyQ, GamepadButton::LeftTrigger],
                ),
                (
                    Action::<TraverseAction>::new(),
                    bindings![KeyCode::KeyE, GamepadButton::RightTrigger],
                ),
            ]),
        ))
        .id();

    commands.entity(player).with_children(|parent| {
        parent.spawn((
            Name::new("Lab Body"),
            LabBody,
            Mesh3d(meshes.add(Capsule3d::new(0.38, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.92, 0.44, 0.24),
                perceptual_roughness: 0.82,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.9, 0.0),
        ));
    });
    spawn_look_rig(commands, meshes, materials, player);

    player
}

fn build_lab_animation_definition() -> CharacterStateMachineDefinition {
    CharacterStateMachineDefinition::new("controller_lab_stack", "Idle")
        .with_fallback_state("Idle")
        .add_state(StateDefinition::new("Idle").with_binding("idle"))
        .add_state(StateDefinition::new("Move").with_binding("move"))
        .add_state(StateDefinition::new("Airborne").with_binding("airborne"))
        .add_transition(
            TransitionDefinition::switch("idle_to_move", "Idle", "Move")
                .when(TransitionCondition::SpeedAtLeast(0.2)),
        )
        .add_transition(
            TransitionDefinition::switch("move_to_idle", "Move", "Idle")
                .when(TransitionCondition::SpeedAtMost(0.1)),
        )
        .add_transition(
            TransitionDefinition::switch("idle_to_airborne", "Idle", "Airborne")
                .when(TransitionCondition::Grounded(false)),
        )
        .add_transition(
            TransitionDefinition::switch("move_to_airborne", "Move", "Airborne")
                .when(TransitionCondition::Grounded(false)),
        )
        .add_transition(
            TransitionDefinition::switch("airborne_to_idle", "Airborne", "Idle")
                .when(TransitionCondition::Grounded(true))
                .when(TransitionCondition::SpeedAtMost(0.1)),
        )
        .add_transition(
            TransitionDefinition::switch("airborne_to_move", "Airborne", "Move")
                .when(TransitionCondition::Grounded(true))
                .when(TransitionCondition::SpeedAtLeast(0.2)),
        )
}

fn spawn_look_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Name::new("Lab Look Target"),
        LabLookTarget,
        Mesh3d(meshes.add(Sphere::new(0.18).mesh().uv(20, 12))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.58, 0.26),
            emissive: Color::srgb(1.0, 0.32, 0.14).into(),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.8, 9.0),
    ));
}

fn spawn_look_rig(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    player: Entity,
) {
    let joint = IkJoint {
        tip_axis: -Vec3::Z,
        pole_axis: Vec3::Y,
        ..default()
    };
    let joint_mesh = meshes.add(Sphere::new(0.12).mesh().uv(20, 12));
    let joint_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.82, 0.96),
        emissive: Color::srgb(0.08, 0.24, 0.32).into(),
        perceptual_roughness: 0.72,
        ..default()
    });

    let root = commands
        .spawn((
            Name::new("Lab Neck Root"),
            Mesh3d(joint_mesh.clone()),
            MeshMaterial3d(joint_material.clone()),
            Transform::from_xyz(0.0, 1.45, -0.05),
            joint,
        ))
        .id();
    let neck = commands
        .spawn((
            Name::new("Lab Neck Joint"),
            Mesh3d(joint_mesh.clone()),
            MeshMaterial3d(joint_material.clone()),
            Transform::from_xyz(0.0, 0.0, -0.24),
            joint,
        ))
        .id();
    let head = commands
        .spawn((
            Name::new("Lab Head Joint"),
            Mesh3d(joint_mesh),
            MeshMaterial3d(joint_material),
            Transform::from_xyz(0.0, 0.0, -0.24),
            joint,
        ))
        .id();

    commands.entity(player).add_child(root);
    commands.entity(root).add_child(neck);
    commands.entity(neck).add_child(head);
    commands.entity(root).insert(IkConstraint::Cone {
        axis: -Vec3::Z,
        max_angle: 1.05,
        strength: 1.0,
    });
    commands.entity(neck).insert(IkConstraint::Cone {
        axis: -Vec3::Z,
        max_angle: 0.85,
        strength: 1.0,
    });

    commands.spawn((
        Name::new("Lab Look Controller"),
        LabLookController,
        IkChain {
            joints: vec![root, neck, head],
            ..default()
        },
        LookAtTarget {
            point: Vec3::new(0.0, 1.8, 9.0),
            forward_axis: -Vec3::Z,
            up_axis: Vec3::Y,
            reach_distance: Some(4.8),
            ..default()
        },
    ));
}

fn follow_camera(
    target: Single<(&Transform, &CharacterControllerState), With<LabController>>,
    mut camera_query: Query<(&LabCamera, &mut Transform), Without<LabController>>,
) {
    let (target_transform, state) = *target;
    let Ok((camera, mut transform)) = camera_query.single_mut() else {
        return;
    };
    let focus = target_transform.translation + Vec3::Y * (camera.height * 0.6);
    let backward = state.orientation * Vec3::Z;
    *transform = Transform::from_translation(
        focus + backward * camera.distance + Vec3::Y * (camera.height * 0.5),
    )
    .looking_at(focus, Vec3::Y);
}

fn sync_animation_facts(
    controller: Single<
        (
            &CharacterController,
            &CharacterControllerState,
            &CharacterMotionStats,
            &LinearVelocity,
            &mut CharacterAnimationFacts,
        ),
        With<LabController>,
    >,
) {
    let (controller, state, stats, velocity, mut facts) = controller.into_inner();
    let normalized_speed = (stats.horizontal_speed / controller.speed.max(0.01)).clamp(0.0, 1.5);
    let facing = state.orientation * -Vec3::Z;

    facts.speed = normalized_speed;
    facts.locomotion_intensity = normalized_speed;
    facts.locomotion_mode = if stats.horizontal_speed > controller.speed * 0.45 {
        LocomotionMode::Run
    } else if stats.horizontal_speed > 0.2 {
        LocomotionMode::Walk
    } else {
        LocomotionMode::Idle
    };
    facts.grounded = state.ground.is_some();
    facts.vertical_velocity = velocity.y;
    facts.facing_direction = Vec2::new(facing.x, facing.z).normalize_or_zero();
    facts.aim_direction = facts.facing_direction;
}

fn sync_look_target(
    time: Res<Time>,
    controller: Single<(&Transform, &CharacterControllerState), With<LabController>>,
    mut target: Single<&mut Transform, (With<LabLookTarget>, Without<LabController>)>,
    mut look_controller: Single<&mut LookAtTarget, With<LabLookController>>,
) {
    let (controller_transform, controller_state) = *controller;
    let forward = controller_state.orientation * -Vec3::Z;
    let right = controller_state.orientation * Vec3::X;
    let bob = (time.elapsed_secs() * 1.2).sin() * 0.35;
    let sway = (time.elapsed_secs() * 0.8).cos() * 0.7;
    let point = controller_transform.translation
        + Vec3::Y * 1.7
        + forward * look_controller.reach_distance.unwrap_or(4.8)
        + right * sway
        + Vec3::Y * bob;
    target.translation = point;
    look_controller.point = point;
}

fn sync_body_visuals(
    controller: Single<
        (
            &CharacterControllerState,
            &CharacterStateMachineRuntime,
            &CharacterAnimationSelection,
        ),
        With<LabController>,
    >,
    look_state: Single<&IkChainState, With<LabLookController>>,
    body: Single<
        (&MeshMaterial3d<StandardMaterial>, &mut Transform),
        (With<LabBody>, Without<LabController>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (state, runtime, selection) = *controller;
    let current_state = runtime
        .current_state
        .as_ref()
        .map(|value| value.0.as_str())
        .unwrap_or("none");
    let current_binding = selection
        .binding
        .as_ref()
        .map(|value| value.0.as_str())
        .unwrap_or("none");
    let (material_handle, mut body_transform) = body.into_inner();
    if let Some(material) = materials.get_mut(&material_handle.0) {
        match current_binding {
            "move" => {
                material.base_color = Color::srgb(0.26, 0.78, 0.46);
                material.emissive = Color::srgb(0.04, 0.10, 0.05).into();
            }
            "airborne" => {
                material.base_color = Color::srgb(0.95, 0.58, 0.24);
                material.emissive = Color::srgb(0.15, 0.08, 0.03).into();
            }
            _ => {
                material.base_color = Color::srgb(0.92, 0.44, 0.24);
                material.emissive = Color::BLACK.into();
            }
        }
    }

    body_transform.translation = Vec3::new(0.0, 0.9, 0.0);
    body_transform.rotation = state.orientation;
    body_transform.scale = match current_state {
        "Airborne" => Vec3::new(0.92, 1.12, 0.92),
        "Move" => Vec3::new(1.06, 0.96, 1.02),
        _ => Vec3::ONE,
    };

    if look_state.last_error > 0.2 {
        body_transform.translation.y += 0.03;
    }
}

fn sync_overlay(
    controller: Single<
        (
            &Transform,
            &CharacterControllerState,
            &CharacterMotionStats,
            &AccumulatedInput,
            &saddle_character_controller::EnvironmentModifiers,
        ),
        With<LabController>,
    >,
    animation: Single<
        (&CharacterStateMachineRuntime, &CharacterAnimationSelection),
        With<LabController>,
    >,
    look_state: Single<&IkChainState, With<LabLookController>>,
    mut overlay: Single<&mut Text, With<LabOverlay>>,
    mut diagnostics: ResMut<LabDiagnostics>,
    names: Query<&Name>,
) {
    let (transform, state, stats, input, env) = *controller;
    let (runtime, selection) = *animation;
    let text = &mut *overlay;

    let ground_name = state
        .ground
        .and_then(|ground| names.get(ground.entity).ok())
        .map_or_else(|| "none".to_owned(), |name| name.as_str().to_owned());
    let support_name = state
        .ground
        .map(|ground| ground.entity)
        .and_then(|entity| names.get(entity).ok())
        .map_or_else(|| "none".to_owned(), |name| name.as_str().to_owned());
    let jump_buffer = input
        .jump_pressed_for
        .map_or_else(|| "none".to_owned(), |age| format!("{age:.2}s"));
    let traverse_buffer = input
        .traverse_pressed_for
        .map_or_else(|| "none".to_owned(), |age| format!("{age:.2}s"));

    text.0.clear();
    let _ = writeln!(text.0, "Character Controller Lab");
    let _ = writeln!(
        text.0,
        "controls: WASD move | mouse look | Space jump | E traverse | Q ascend | Left Click lock | Esc unlock"
    );
    let _ = writeln!(text.0, "pane: tweak controller, flight, and IK live");
    let _ = writeln!(text.0, "mode: {:?}", state.movement_mode);
    let _ = writeln!(text.0, "ground: {ground_name}");
    let _ = writeln!(
        text.0,
        "support: {support_name} lin={:.2} ang={:.2}",
        state.support_velocity.length(),
        state.support_angular_velocity.length()
    );
    let _ = writeln!(
        text.0,
        "env: {:?} speed={:.2} accel={:.2} gravity={:.2}",
        env.depth, env.speed_multiplier, env.acceleration_multiplier, env.gravity_multiplier
    );
    let _ = writeln!(
        text.0,
        "pos: ({:.1}, {:.1}, {:.1}) speed={:.2} horiz={:.2}",
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
        stats.current_speed,
        stats.horizontal_speed
    );
    let _ = writeln!(text.0, "casts: {}", stats.shape_casts_last_tick);
    let _ = writeln!(
        text.0,
        "anim: {} / {} look_err={:.3}",
        runtime
            .current_state
            .as_ref()
            .map(|value| value.0.as_str())
            .unwrap_or("none"),
        selection
            .binding
            .as_ref()
            .map(|value| value.0.as_str())
            .unwrap_or("none"),
        look_state.last_error
    );
    let _ = write!(
        text.0,
        "buffers: jump={jump_buffer} traverse={traverse_buffer}"
    );

    diagnostics.controller_position = transform.translation;
    diagnostics.movement_mode = format!("{:?}", state.movement_mode);
    diagnostics.grounded_on = support_name;
    diagnostics.support_angular_speed = state.support_angular_velocity.length();
    diagnostics.current_speed = stats.current_speed;
    diagnostics.animation_state = runtime
        .current_state
        .as_ref()
        .map(|value| value.0.clone())
        .unwrap_or_else(|| "none".into());
    diagnostics.animation_binding = selection
        .binding
        .as_ref()
        .map(|value| value.0.clone())
        .unwrap_or_else(|| "none".into());
    diagnostics.look_error = look_state.last_error;
}

fn sync_lab_pane(
    mut pane: ResMut<LabPane>,
    mut controller: Single<
        (
            &mut CharacterController,
            &mut CharacterLook,
            &mut CharacterFlying,
            &CharacterControllerState,
            &CharacterMotionStats,
        ),
        With<LabController>,
    >,
    mut look_controller: Single<(&mut LookAtTarget, &IkChainState), With<LabLookController>>,
    animation: Single<
        (&CharacterStateMachineRuntime, &CharacterAnimationSelection),
        With<LabController>,
    >,
    names: Query<&Name>,
) {
    if pane.is_changed() && !pane.is_added() {
        controller.0.speed = pane.speed;
        controller.0.sprint_speed_scale = pane.sprint_speed_scale;
        controller.0.jump_height = pane.jump_height;
        controller.0.step_size = pane.step_size;
        controller.0.support_rotation_policy = if pane.inherit_support_yaw {
            SupportRotationPolicy::YawOnly
        } else {
            SupportRotationPolicy::None
        };
        controller.1.sensitivity = Vec2::splat(pane.look_sensitivity);
        controller.2.enabled = pane.flight_enabled;
        controller.2.speed = pane.flight_speed;
        controller.2.collision_mode = if pane.flight_noclip {
            FlightCollisionMode::NoClip
        } else {
            FlightCollisionMode::Slide
        };
        look_controller.0.reach_distance = Some(pane.look_distance);
        look_controller.0.weight.rotation = pane.look_weight;
    }

    let grounded_on = controller
        .3
        .ground
        .and_then(|ground| names.get(ground.entity).ok())
        .map_or_else(|| "none".to_owned(), |name| name.as_str().to_owned());

    let pane = pane.bypass_change_detection();
    pane.speed = controller.0.speed;
    pane.sprint_speed_scale = controller.0.sprint_speed_scale;
    pane.jump_height = controller.0.jump_height;
    pane.step_size = controller.0.step_size;
    pane.look_sensitivity = controller.1.sensitivity.x;
    pane.inherit_support_yaw =
        controller.0.support_rotation_policy == SupportRotationPolicy::YawOnly;
    pane.flight_enabled = controller.2.enabled;
    pane.flight_speed = controller.2.speed;
    pane.flight_noclip = controller.2.collision_mode == FlightCollisionMode::NoClip;
    pane.look_distance = look_controller.0.reach_distance.unwrap_or(0.0);
    pane.look_weight = look_controller.0.weight.rotation;
    pane.movement_mode = format!("{:?}", controller.3.movement_mode);
    pane.grounded_on = grounded_on;
    pane.current_speed = controller.4.current_speed;
    pane.support_angular_speed = controller.3.support_angular_velocity.length();
    pane.animation_state = animation
        .0
        .current_state
        .as_ref()
        .map(|value| value.0.clone())
        .unwrap_or_else(|| "none".into());
    pane.animation_binding = animation
        .1
        .binding
        .as_ref()
        .map(|value| value.0.clone())
        .unwrap_or_else(|| "none".into());
    pane.look_error = look_controller.1.last_error;
}

fn capture_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::None;
    cursor.visible = true;
}

fn animate_platforms(
    time: Res<Time<Fixed>>,
    mut query: Query<(
        &MovingPlatform,
        &mut Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
    )>,
) {
    let t = time.elapsed_secs();
    for (platform, mut transform, mut linear_velocity, mut angular_velocity) in &mut query {
        let axis = platform.translation_axis.normalize_or_zero();
        let phase = t * platform.translation_speed + platform.phase;
        let displacement = axis * (phase.sin() * platform.translation_amplitude);
        transform.translation = platform.origin + displacement;
        linear_velocity.0 =
            axis * (phase.cos() * platform.translation_amplitude * platform.translation_speed);
        transform.rotation = Quat::from_rotation_y(t * platform.rotation_speed);
        angular_velocity.0 = Vec3::Y * platform.rotation_speed;
    }
}

fn spawn_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    color: Color,
) {
    commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn spawn_dynamic_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    color: Color,
) {
    commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.82,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Dynamic,
        Collider::cuboid(size.x, size.y, size.z),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
    ));
}

fn spawn_ramp(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    tilt_z: f32,
    color: Color,
    surface: Option<MovementSurface>,
) {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.84,
            ..default()
        })),
        Transform::from_translation(center).with_rotation(Quat::from_rotation_z(tilt_z)),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
    if let Some(surface) = surface {
        entity.insert(surface);
    }
}

fn spawn_stairs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    start: Vec3,
    steps: usize,
    depth: f32,
    height: f32,
    width: f32,
) {
    for step in 0..steps {
        let size = Vec3::new(width, height * (step as f32 + 1.0), depth);
        let center = start + Vec3::new(0.0, size.y * 0.5, -(step as f32 * depth) - depth * 0.5);
        spawn_block(
            commands,
            meshes,
            materials,
            &format!("Lab Step {step}"),
            center,
            size,
            Color::srgb(0.34, 0.41, 0.52),
        );
    }
}

fn spawn_platform(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
    color: Color,
    moving: Option<MovingPlatform>,
    surface: Option<MovementSurface>,
) {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.05,
            perceptual_roughness: 0.75,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Kinematic,
        Collider::cuboid(size.x, size.y, size.z),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
    ));
    if let Some(moving) = moving {
        entity.insert(moving);
    }
    if let Some(surface) = surface {
        entity.insert(surface);
    }
}

fn spawn_water_volume(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    center: Vec3,
    size: Vec3,
) {
    commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.18, 0.46, 0.72, 0.38),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.2,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
        SwimVolume {
            speed_multiplier: 0.55,
            acceleration_multiplier: 0.8,
            gravity_multiplier: 0.4,
        },
    ));
}
