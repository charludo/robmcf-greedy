mod camera;
mod network;

use camera::CameraPlugin;
use network::NetworkPlugin;

use bevy::{prelude::*, window::PresentMode};
use bevy_mod_picking::{debug::DebugPickingPlugin, DefaultPickingPlugins, PickableBundle};

fn main() {
    let height: f32 = 900.0;
    let resolution: f32 = 16.0 / 9.0;
    let width: f32 = height * resolution;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RobMCF Greedy".to_string(),
                resolution: (width, height).into(),
                mode: bevy::window::WindowMode::BorderlessFullscreen,
                present_mode: PresentMode::AutoVsync,
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.16, 0.16, 0.18)))
        .add_plugins(CameraPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        )
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(PickableBundle::default());
}
