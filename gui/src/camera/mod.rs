use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_picking::PickableBundle;
use simple_easing::cubic_in_out;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (
                start_dragging,
                drag,
                stop_dragging,
                set_zoom_scale,
                animate_zoom_scale,
                update_world_coords,
            ),
        );
    }
}

#[derive(Component)]
pub struct Zoom {
    start: f32,
    pub target: f32,
    timer: Timer,
}

#[derive(Component)]
pub struct CameraMarker;

#[derive(Resource, Default)]
pub struct WorldCoords(pub Vec2);

#[derive(Component)]
pub struct BackgroundMarker;

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn((
            Camera2dBundle::default(),
            Zoom {
                start: 10.0,
                target: 3.0,
                timer: Timer::from_seconds(0.4, TimerMode::Once),
            },
            CameraMarker,
        ))
        .with_children(|child| {
            child.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(0., 0., 0., 0.),
                        custom_size: Some(Vec2::MAX),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(0., 0., -1.),
                    ..Default::default()
                },
                PickableBundle::default(),
                BackgroundMarker,
            ));
        });
    commands.insert_resource(WorldCoords::default());
}

fn update_world_coords(
    mut world_coords: ResMut<WorldCoords>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
) {
    let (camera, camera_transform) = query_camera.single();
    let window = query_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        world_coords.0 = world_position;
    }
}

#[derive(Component)]
struct Dragging {
    origin: Vec2,
}

fn start_dragging(
    mut commands: Commands,
    keys: Res<ButtonInput<MouseButton>>,
    query_camera: Query<Entity, (With<Camera>, Without<Dragging>)>,
    windows: Query<&Window>,
) {
    if !keys.pressed(MouseButton::Middle) {
        return;
    }

    if let Some(cursor_position) = windows.single().cursor_position() {
        if let Ok(camera) = query_camera.get_single() {
            commands.entity(camera).insert(Dragging {
                origin: cursor_position,
            });
        }
    }
}

fn stop_dragging(
    mut commands: Commands,
    keys: Res<ButtonInput<MouseButton>>,
    query_camera: Query<Entity, With<Dragging>>,
) {
    if keys.pressed(MouseButton::Middle) {
        return;
    }
    if let Ok(camera) = query_camera.get_single() {
        commands.entity(camera).remove::<Dragging>();
    }
}

fn drag(
    mut query_camera: Query<(&mut Transform, &Zoom, &mut Dragging), With<Dragging>>,
    windows: Query<&Window>,
) {
    if let Some(cursor_position) = windows.single().cursor_position() {
        if let Ok((mut transform, zoom, mut dragging)) = query_camera.get_single_mut() {
            let delta = cursor_position - dragging.origin;
            transform.translation += delta
                .mul_add(Vec2::new(-zoom.target, zoom.target), Vec2::new(0., 0.))
                .extend(0.);
            dragging.origin = cursor_position;
        }
    }
}

fn set_zoom_scale(
    mut query: Query<(&mut Zoom, &OrthographicProjection)>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    let (mut zoom, projection) = query.single_mut();
    for event in scroll_evr.read() {
        zoom.start = projection.scale;
        zoom.timer.reset();
        if event.y.is_sign_positive() {
            zoom.target /= 1.5;
        } else {
            zoom.target *= 1.5;
        }
    }
}

fn animate_zoom_scale(
    time: Res<Time<Real>>,
    mut query: Query<(&mut Zoom, &mut OrthographicProjection)>,
) {
    let (mut zoom, mut projection) = query.single_mut();
    if !zoom.timer.finished() {
        zoom.timer
            .tick(std::time::Duration::from_secs_f32(time.delta_seconds()));
        projection.scale =
            zoom.start + cubic_in_out(zoom.timer.fraction()) * (zoom.target - zoom.start);
    }
}
