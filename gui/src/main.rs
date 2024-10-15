use bevy::prelude::*;
use bevy::window::PresentMode;
// use bevy_mod_picking::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_prototype_lyon::prelude::*;
use robmcf_greedy::{Network, Options};
use shared::*;

mod camera;
mod network;
mod shared;

use camera::CameraPlugin;
use network::NetworkPlugin;

#[derive(Resource)]
struct NetworkWrapper {
    n: Network,
    num_vertices: usize,
}

fn main() {
    let n = Network::from_file(&Options::default(), "examples/cologne.json")
        .expect("An error occurred while loading the network.");
    let network = NetworkWrapper {
        num_vertices: n.vertices.len(),
        n,
    };
    println!("{}", network.n);

    let app_settings = AppSettings {
        background_color: Color::srgb(0.16, 0.16, 0.18),
        highlight_color: Color::srgb(0.38, 0.58, 0.78),
        baseline_color: Color::WHITE,

        vertex_layer: 3.0,
        vertex_selected_layer: 4.0,
        arc_layer: 0.0,
        arc_fixed_layer: 1.0,
        arc_selected_layer: 2.0,
    };

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
        .insert_resource(ClearColor(app_settings.background_color))
        .insert_resource(network)
        .insert_resource(app_settings)
        .add_plugins(ShapePlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(DefaultPickingPlugins)
        // .insert_resource(DebugPickingMode::Normal)
        .run();
}
