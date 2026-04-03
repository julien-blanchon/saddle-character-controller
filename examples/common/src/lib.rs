use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, WindowResolution},
};
use bevy_enhanced_input::prelude::*;
use saddle_character_controller::{
    AccumulatedInput, AscendAction, CharacterController, CharacterControllerDebugDraw,
    CharacterControllerPlugin, CharacterControllerState, CharacterControllerSystems,
    CharacterFlying, CharacterLook, CharacterMantle, CharacterMotionStats, CharacterPush,
    CharacterSwimming, CharacterWallKick, CrouchAction, FlightCollisionMode, JumpAction,
    LookAction, MoveAction, MovementSurface, SprintAction, SupportRotationPolicy,
    SupportVelocityPolicy, TraverseAction, WaterVolume,
};
use saddle_pane::prelude::*;

#[derive(Clone, Copy, Debug, Resource)]
pub struct DemoConfig {
    pub title: &'static str,
    pub scene: DemoScene,
    pub camera_mode: CameraMode,
    pub debug_draw: bool,
    pub controller_count: usize,
}

impl DemoConfig {
    pub fn basic() -> Self {
        Self {
            title: "character_controller basic",
            scene: DemoScene::Basic,
            camera_mode: CameraMode::FirstPerson,
            debug_draw: false,
            controller_count: 1,
        }
    }

    pub fn slopes_and_stairs() -> Self {
        Self {
            title: "character_controller slopes_and_stairs",
            scene: DemoScene::SlopesAndStairs,
            camera_mode: CameraMode::FirstPerson,
            debug_draw: false,
            controller_count: 1,
        }
    }

    pub fn moving_platforms() -> Self {
        Self {
            title: "character_controller moving_platforms",
            scene: DemoScene::MovingPlatforms,
            camera_mode: CameraMode::FirstPerson,
            debug_draw: false,
            controller_count: 1,
        }
    }

    pub fn advanced_movement() -> Self {
        Self {
            title: "character_controller advanced_movement",
            scene: DemoScene::AdvancedMovement,
            camera_mode: CameraMode::FirstPerson,
            debug_draw: true,
            controller_count: 1,
        }
    }

    pub fn traversal() -> Self {
        Self {
            title: "character_controller traversal",
            scene: DemoScene::Traversal,
            camera_mode: CameraMode::FirstPerson,
            debug_draw: true,
            controller_count: 1,
        }
    }

    pub fn water() -> Self {
        Self {
            title: "character_controller water",
            scene: DemoScene::Water,
            camera_mode: CameraMode::FirstPerson,
            debug_draw: false,
            controller_count: 1,
        }
    }

    pub fn third_person() -> Self {
        Self {
            title: "character_controller third_person",
            scene: DemoScene::ThirdPerson,
            camera_mode: CameraMode::ThirdPerson,
            debug_draw: false,
            controller_count: 1,
        }
    }

    pub fn stress_many_controllers() -> Self {
        Self {
            title: "character_controller stress_many_controllers",
            scene: DemoScene::StressManyControllers,
            camera_mode: CameraMode::Overview,
            debug_draw: false,
            controller_count: 50,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoScene {
    Basic,
    SlopesAndStairs,
    MovingPlatforms,
    AdvancedMovement,
    Traversal,
    Water,
    ThirdPerson,
    StressManyControllers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
    Overview,
}

#[derive(Component)]
struct FirstPersonCamera {
    target: Entity,
}

#[derive(Component)]
struct DemoPlayer;

#[derive(Component)]
struct ThirdPersonCamera {
    target: Entity,
    distance: f32,
    height: f32,
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
enum DemoFixedSystems {
    AnimatePlatforms,
}

#[derive(Resource, Pane)]
#[pane(title = "Character Controller")]
struct ControllerPane {
    #[pane(tab = "Movement", slider, min = 4.0, max = 20.0, step = 0.1)]
    speed: f32,
    #[pane(tab = "Movement", slider, min = 1.0, max = 2.5, step = 0.05)]
    sprint_speed_scale: f32,
    #[pane(tab = "Movement", slider, min = 0.5, max = 3.0, step = 0.05)]
    jump_height: f32,
    #[pane(tab = "Movement", slider, min = 0.1, max = 1.2, step = 0.05)]
    step_size: f32,
    #[pane(tab = "Movement", slider, min = 0.0, max = 24.0, step = 0.5)]
    air_acceleration_hz: f32,
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
    #[pane(tab = "Runtime", monitor)]
    movement_mode: String,
    #[pane(tab = "Runtime", monitor)]
    grounded: bool,
    #[pane(tab = "Runtime", monitor)]
    current_speed: f32,
    #[pane(tab = "Runtime", monitor)]
    support_entity: String,
}

impl Default for ControllerPane {
    fn default() -> Self {
        Self {
            speed: 11.0,
            sprint_speed_scale: 1.5,
            jump_height: 1.8,
            step_size: 0.7,
            air_acceleration_hz: 12.0,
            look_sensitivity: 0.0022,
            inherit_support_yaw: true,
            flight_enabled: false,
            flight_speed: 14.0,
            flight_noclip: false,
            movement_mode: "Airborne".into(),
            grounded: false,
            current_speed: 0.0,
            support_entity: "None".into(),
        }
    }
}

pub fn pane_plugins() -> (
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

pub fn run_demo(config: DemoConfig) -> AppExit {
    let mut app = App::new();
    app.insert_resource(config)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: config.title.into(),
                    resolution: WindowResolution::new(1440, 900),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            CharacterControllerPlugin::always_on(FixedUpdate),
            pane_plugins(),
        ))
        .register_pane::<ControllerPane>()
        .configure_sets(
            FixedUpdate,
            DemoFixedSystems::AnimatePlatforms.before(CharacterControllerSystems::Grounding),
        )
        .add_systems(Startup, setup_scene)
        .add_systems(
            FixedUpdate,
            animate_platforms.in_set(DemoFixedSystems::AnimatePlatforms),
        )
        .add_systems(
            Update,
            (
                capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
                release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                sync_controller_pane,
            ),
        )
        .add_systems(
            PostUpdate,
            (follow_first_person_camera, follow_third_person_camera),
        );

    if config.debug_draw {
        app.insert_resource(CharacterControllerDebugDraw {
            enabled: true,
            ..default()
        });
    }

    app.run()
}

fn setup_scene(
    mut commands: Commands,
    config: Res<DemoConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_lighting(&mut commands);

    let player = match config.scene {
        DemoScene::Basic => {
            spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 90.0);
            spawn_basic_props(&mut commands, &mut meshes, &mut materials);
            spawn_primary_controller(
                &mut commands,
                &mut meshes,
                &mut materials,
                Transform::from_xyz(0.0, 3.0, 14.0),
                config.camera_mode,
                config.scene,
            )
        }
        DemoScene::SlopesAndStairs => {
            spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 120.0);
            spawn_slopes_course(&mut commands, &mut meshes, &mut materials);
            spawn_primary_controller(
                &mut commands,
                &mut meshes,
                &mut materials,
                Transform::from_xyz(-14.0, 3.0, 12.0),
                config.camera_mode,
                config.scene,
            )
        }
        DemoScene::MovingPlatforms => {
            spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 120.0);
            spawn_platform_course(&mut commands, &mut meshes, &mut materials);
            spawn_primary_controller(
                &mut commands,
                &mut meshes,
                &mut materials,
                Transform::from_xyz(-16.0, 3.0, 0.0),
                config.camera_mode,
                config.scene,
            )
        }
        DemoScene::AdvancedMovement => {
            spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 140.0);
            spawn_advanced_course(&mut commands, &mut meshes, &mut materials);
            spawn_primary_controller(
                &mut commands,
                &mut meshes,
                &mut materials,
                Transform::from_xyz(-20.0, 3.0, 6.0),
                config.camera_mode,
                config.scene,
            )
        }
        DemoScene::Traversal => {
            spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 120.0);
            spawn_traversal_course(&mut commands, &mut meshes, &mut materials);
            spawn_primary_controller(
                &mut commands,
                &mut meshes,
                &mut materials,
                Transform::from_xyz(-12.0, 3.0, 12.0),
                config.camera_mode,
                config.scene,
            )
        }
        DemoScene::Water => {
            spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 120.0);
            spawn_water_course(&mut commands, &mut meshes, &mut materials);
            spawn_primary_controller(
                &mut commands,
                &mut meshes,
                &mut materials,
                Transform::from_xyz(-14.0, 3.0, 10.0),
                config.camera_mode,
                config.scene,
            )
        }
        DemoScene::ThirdPerson => {
            spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 120.0);
            spawn_basic_props(&mut commands, &mut meshes, &mut materials);
            spawn_traversal_course(&mut commands, &mut meshes, &mut materials);
            spawn_primary_controller(
                &mut commands,
                &mut meshes,
                &mut materials,
                Transform::from_xyz(0.0, 3.0, 12.0),
                config.camera_mode,
                config.scene,
            )
        }
        DemoScene::StressManyControllers => {
            spawn_flat_ground(&mut commands, &mut meshes, &mut materials, 180.0);
            spawn_stress_obstacles(&mut commands, &mut meshes, &mut materials);
            let player = spawn_primary_controller(
                &mut commands,
                &mut meshes,
                &mut materials,
                Transform::from_xyz(0.0, 3.0, 16.0),
                CameraMode::Overview,
                config.scene,
            );
            spawn_extra_controllers(
                &mut commands,
                &mut meshes,
                &mut materials,
                config.controller_count.saturating_sub(1),
            );
            player
        }
    };

    spawn_camera(&mut commands, player, config.camera_mode);
}

fn spawn_primary_controller(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    transform: Transform,
    camera_mode: CameraMode,
    scene: DemoScene,
) -> Entity {
    let mut controller = CharacterController {
        speed: 11.0,
        jump_input_buffer: Duration::from_millis(160),
        coyote_time: Duration::from_millis(110),
        ..default()
    };

    let mut swimming = None;
    let mut flying = Some(CharacterFlying::default());
    let mut mantle = None;
    let mut wall_kick = None;
    let mut push = Some(CharacterPush::default());
    let render_body = matches!(camera_mode, CameraMode::ThirdPerson | CameraMode::Overview);

    match scene {
        DemoScene::MovingPlatforms => {
            controller.support_velocity_policy = SupportVelocityPolicy::Full;
            controller.support_detach_grace = Duration::from_millis(180);
        }
        DemoScene::AdvancedMovement => {
            controller.speed = 14.0;
            controller.friction_hz = 7.0;
            controller.air_acceleration_hz = 18.0;
            controller.max_air_wish_speed = 0.95;
            controller.auto_bhop = true;
            controller.slide_gravity_scale = 1.2;
            mantle = Some(CharacterMantle::default());
            wall_kick = Some(CharacterWallKick::default());
        }
        DemoScene::Traversal => {
            controller.jump_height = 2.1;
            controller.step_size = 0.8;
            controller.coyote_time = Duration::from_millis(140);
            mantle = Some(CharacterMantle::default());
            wall_kick = Some(CharacterWallKick::default());
        }
        DemoScene::Water => {
            swimming = Some(CharacterSwimming::default());
        }
        DemoScene::ThirdPerson => {
            controller.speed = 10.0;
            controller.sprint_speed_scale = 1.35;
            swimming = Some(CharacterSwimming::default());
            mantle = Some(CharacterMantle::default());
        }
        DemoScene::StressManyControllers => {
            controller.speed = 9.0;
            push = None;
        }
        DemoScene::Basic | DemoScene::SlopesAndStairs => {}
    }

    let look = CharacterLook {
        sensitivity: Vec2::splat(0.0022),
        ..default()
    };

    let mut entity = commands.spawn((
        Name::new("Player"),
        DemoPlayer,
        controller,
        look,
        Visibility::Inherited,
        transform,
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
    ));

    if let Some(swimming) = swimming {
        entity.insert(swimming);
    }
    if let Some(flying) = flying.take() {
        entity.insert(flying);
    }
    if let Some(mantle) = mantle {
        entity.insert(mantle);
    }
    if let Some(wall_kick) = wall_kick {
        entity.insert(wall_kick);
    }
    if let Some(push) = push {
        entity.insert(push);
    }

    let player = entity.id();
    if render_body {
        spawn_controller_visual(
            commands,
            meshes,
            materials,
            player,
            Color::srgb(0.92, 0.44, 0.22),
        );
    }

    player
}

fn sync_controller_pane(
    mut pane: ResMut<ControllerPane>,
    mut players: Query<
        (
            &mut CharacterController,
            &mut CharacterLook,
            &CharacterControllerState,
            &CharacterMotionStats,
            Option<&mut CharacterFlying>,
        ),
        With<DemoPlayer>,
    >,
) {
    let Ok((mut controller, mut look, state, stats, flying)) = players.single_mut() else {
        return;
    };
    let mut flight_snapshot = None;

    if pane.is_changed() && !pane.is_added() {
        controller.speed = pane.speed;
        controller.sprint_speed_scale = pane.sprint_speed_scale;
        controller.jump_height = pane.jump_height;
        controller.step_size = pane.step_size;
        controller.air_acceleration_hz = pane.air_acceleration_hz;
        controller.support_rotation_policy = if pane.inherit_support_yaw {
            SupportRotationPolicy::YawOnly
        } else {
            SupportRotationPolicy::None
        };
        look.sensitivity = Vec2::splat(pane.look_sensitivity);
    }

    if let Some(mut flying) = flying {
        if pane.is_changed() && !pane.is_added() {
            flying.enabled = pane.flight_enabled;
            flying.speed = pane.flight_speed;
            flying.collision_mode = if pane.flight_noclip {
                FlightCollisionMode::NoClip
            } else {
                FlightCollisionMode::Slide
            };
        }
        flight_snapshot = Some((
            flying.enabled,
            flying.speed,
            matches!(flying.collision_mode, FlightCollisionMode::NoClip),
        ));
    }

    let pane = pane.bypass_change_detection();
    pane.speed = controller.speed;
    pane.sprint_speed_scale = controller.sprint_speed_scale;
    pane.jump_height = controller.jump_height;
    pane.step_size = controller.step_size;
    pane.air_acceleration_hz = controller.air_acceleration_hz;
    pane.look_sensitivity = look.sensitivity.x;
    pane.inherit_support_yaw = controller.support_rotation_policy == SupportRotationPolicy::YawOnly;
    if let Some((enabled, speed, noclip)) = flight_snapshot {
        pane.flight_enabled = enabled;
        pane.flight_speed = speed;
        pane.flight_noclip = noclip;
    } else {
        pane.flight_enabled = false;
        pane.flight_speed = 0.0;
        pane.flight_noclip = false;
    }
    pane.movement_mode = format!("{:?}", state.movement_mode);
    pane.grounded = state.ground.is_some();
    pane.current_speed = stats.current_speed;
    pane.support_entity = stats
        .last_support_entity
        .map(|entity| format!("{entity:?}"))
        .unwrap_or_else(|| "None".into());
}

fn spawn_extra_controllers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    count: usize,
) {
    let mut spawned = 0usize;
    let columns = 10usize;
    let spacing = 4.5;

    for row in 0.. {
        for column in 0..columns {
            if spawned >= count {
                return;
            }
            let x = column as f32 * spacing - 20.0;
            let z = row as f32 * spacing - 12.0;
            let controller = CharacterController {
                speed: 8.0,
                jump_height: 1.4,
                max_speed: 40.0,
                ..default()
            };
            let entity = commands
                .spawn((
                    Name::new(format!("Bot {spawned:02}")),
                    controller,
                    Visibility::Inherited,
                    Transform::from_xyz(x, 3.0, z),
                ))
                .id();
            let tint = Color::srgb(0.25 + 0.02 * column as f32, 0.35 + 0.01 * row as f32, 0.75);
            spawn_controller_visual(commands, meshes, materials, entity, tint);
            spawned += 1;
        }
    }
}

fn spawn_controller_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    entity: Entity,
    color: Color,
) {
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 1.8, 0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.85,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.9, 0.0),
        ));
    });
}

fn spawn_camera(commands: &mut Commands, target: Entity, camera_mode: CameraMode) {
    match camera_mode {
        CameraMode::FirstPerson => {
            commands.spawn((
                Name::new("First Person Camera"),
                Camera3d::default(),
                Projection::Perspective(PerspectiveProjection {
                    fov: std::f32::consts::TAU / 5.5,
                    ..default()
                }),
                FirstPersonCamera { target },
            ));
        }
        CameraMode::ThirdPerson => {
            commands.spawn((
                Name::new("Third Person Camera"),
                Camera3d::default(),
                ThirdPersonCamera {
                    target,
                    distance: 6.5,
                    height: 2.8,
                },
            ));
        }
        CameraMode::Overview => {
            commands.spawn((
                Name::new("Overview Camera"),
                Camera3d::default(),
                Transform::from_xyz(22.0, 28.0, 26.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ));
        }
    }
}

fn follow_first_person_camera(
    targets: Query<
        (&Transform, &CharacterController, &CharacterControllerState),
        Without<FirstPersonCamera>,
    >,
    mut cameras: Query<(&FirstPersonCamera, &mut Transform), Without<CharacterController>>,
) {
    for (camera, mut transform) in &mut cameras {
        let Ok((target_transform, controller, state)) = targets.get(camera.target) else {
            continue;
        };
        let view_height = if state.crouching {
            controller.crouch_view_height
        } else {
            controller.standing_view_height
        };
        transform.translation = target_transform.translation + Vec3::Y * view_height;
        transform.rotation = state.orientation;
    }
}

fn follow_third_person_camera(
    targets: Query<
        (&Transform, &CharacterControllerState),
        (With<CharacterController>, Without<ThirdPersonCamera>),
    >,
    mut cameras: Query<(&ThirdPersonCamera, &mut Transform), Without<CharacterController>>,
) {
    for (camera, mut transform) in &mut cameras {
        let Ok((target_transform, state)) = targets.get(camera.target) else {
            continue;
        };
        let focus = target_transform.translation + Vec3::Y * camera.height * 0.65;
        let backward = state.orientation * Vec3::Z;
        transform.translation =
            focus + backward * camera.distance + Vec3::Y * (camera.height * 0.55);
        *transform = transform.looking_at(focus, Vec3::Y);
    }
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

fn spawn_lighting(commands: &mut Commands) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 24_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(16.0, 22.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_flat_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    size: f32,
) {
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(size)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.17, 0.21, 0.19),
            perceptual_roughness: 1.0,
            ..default()
        })),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));
}

fn spawn_basic_props(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_block(
        commands,
        meshes,
        materials,
        "Crate Cluster A",
        Vec3::new(-4.0, 1.0, 2.0),
        Vec3::new(2.0, 2.0, 2.0),
        Color::srgb(0.56, 0.39, 0.26),
    );
    spawn_block(
        commands,
        meshes,
        materials,
        "Crate Cluster B",
        Vec3::new(4.5, 0.5, -3.0),
        Vec3::new(3.0, 1.0, 3.0),
        Color::srgb(0.39, 0.47, 0.62),
    );
    spawn_block(
        commands,
        meshes,
        materials,
        "Crate Cluster C",
        Vec3::new(0.0, 1.5, -10.0),
        Vec3::new(2.5, 3.0, 2.5),
        Color::srgb(0.31, 0.58, 0.47),
    );
}

fn spawn_slopes_course(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_ramp(
        commands,
        meshes,
        materials,
        "Walkable Ramp",
        Vec3::new(-6.0, 1.2, 0.0),
        Vec3::new(8.0, 0.8, 8.0),
        -0.4,
        Color::srgb(0.23, 0.49, 0.41),
        None,
    );
    spawn_ramp(
        commands,
        meshes,
        materials,
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
    spawn_stairs(
        commands,
        meshes,
        materials,
        Vec3::new(-18.0, 0.0, -5.0),
        7,
        1.2,
        0.25,
        2.0,
    );
}

fn spawn_platform_course(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_platform(
        commands,
        meshes,
        materials,
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
    spawn_platform(
        commands,
        meshes,
        materials,
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
    spawn_platform(
        commands,
        meshes,
        materials,
        "Conveyor Strip",
        Vec3::new(16.0, 0.4, 0.0),
        Vec3::new(8.0, 0.2, 3.0),
        Color::srgb(0.42, 0.37, 0.76),
        None,
        Some(MovementSurface {
            conveyor_velocity: Vec3::new(2.5, 0.0, 0.0),
            inherit_velocity_policy: Some(SupportVelocityPolicy::Horizontal),
            ..default()
        }),
    );
}

fn spawn_advanced_course(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_slopes_course(commands, meshes, materials);
    spawn_platform_course(commands, meshes, materials);
    spawn_ramp(
        commands,
        meshes,
        materials,
        "Surf Wall Left",
        Vec3::new(18.0, 2.2, -10.0),
        Vec3::new(14.0, 0.5, 10.0),
        -1.1,
        Color::srgb(0.76, 0.36, 0.28),
        Some(MovementSurface {
            slide_only: true,
            traction_multiplier: 0.25,
            ..default()
        }),
    );
    spawn_ramp(
        commands,
        meshes,
        materials,
        "Surf Wall Right",
        Vec3::new(18.0, 2.2, 10.0),
        Vec3::new(14.0, 0.5, 10.0),
        1.1,
        Color::srgb(0.28, 0.55, 0.74),
        Some(MovementSurface {
            slide_only: true,
            traction_multiplier: 0.25,
            ..default()
        }),
    );
}

fn spawn_traversal_course(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_block(
        commands,
        meshes,
        materials,
        "Mantle Block",
        Vec3::new(0.0, 0.75, -2.0),
        Vec3::new(2.0, 1.5, 2.0),
        Color::srgb(0.35, 0.58, 0.44),
    );
    spawn_block(
        commands,
        meshes,
        materials,
        "Tall Wall",
        Vec3::new(8.0, 2.0, -6.0),
        Vec3::new(1.0, 4.0, 8.0),
        Color::srgb(0.62, 0.32, 0.28),
    );
    spawn_stairs(
        commands,
        meshes,
        materials,
        Vec3::new(-10.0, 0.0, -8.0),
        6,
        1.0,
        0.3,
        2.0,
    );
    spawn_platform(
        commands,
        meshes,
        materials,
        "Traversal Ledge",
        Vec3::new(14.0, 2.6, 4.0),
        Vec3::new(6.0, 0.3, 3.0),
        Color::srgb(0.44, 0.49, 0.68),
        None,
        None,
    );
}

fn spawn_water_course(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_basic_props(commands, meshes, materials);
    spawn_block(
        commands,
        meshes,
        materials,
        "Pool Rim",
        Vec3::new(6.0, 0.2, 0.0),
        Vec3::new(18.0, 0.4, 14.0),
        Color::srgb(0.36, 0.34, 0.31),
    );
    spawn_water_volume(
        commands,
        meshes,
        materials,
        "Pool",
        Vec3::new(6.0, 1.4, 0.0),
        Vec3::new(12.0, 3.0, 8.0),
    );
}

fn spawn_stress_obstacles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for i in 0..8 {
        let x = -30.0 + i as f32 * 8.0;
        spawn_block(
            commands,
            meshes,
            materials,
            &format!("Stress Pillar {i}"),
            Vec3::new(x, 1.5, -14.0),
            Vec3::new(2.0, 3.0, 2.0),
            Color::srgb(0.4, 0.42, 0.48),
        );
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
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_translation(center),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

#[allow(clippy::too_many_arguments)]
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
            perceptual_roughness: 0.85,
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

#[allow(clippy::too_many_arguments)]
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
            &format!("Step {step}"),
            center,
            size,
            Color::srgb(0.34, 0.41, 0.52),
        );
    }
}

#[allow(clippy::too_many_arguments)]
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
        WaterVolume::default(),
    ));
}

#[allow(dead_code)]
fn _type_smoke(
    _input: &AccumulatedInput,
    _state: &CharacterControllerState,
    _stats: &CharacterMotionStats,
) {
}
