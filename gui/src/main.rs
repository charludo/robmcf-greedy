use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
mod camera;
mod network;

use camera::CameraPlugin;
use network::NetworkPlugin;

use bevy::window::PresentMode;
use bevy_mod_picking::{debug::DebugPickingPlugin, DefaultPickingPlugins, PickableBundle};
use robmcf_greedy::Network;

#[derive(Resource)]
struct NetworkWrapper {
    n: Network,
}

fn main() {
    let network = NetworkWrapper {
        n: Network::from_random(
            20,       // num_vertices,
            0.1,      // connectedness,
            0.3,      // supply_density,
            2,        // num_scenarios,
            (3, 8),   // range_supply,
            (15, 40), // range_capacity,
            (4, 8),   // range_cost,
            5,        // num_fixed_arcs,
        ),
    };
    println!("{}", network.n);

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
        .insert_resource(network)
        .add_plugins(ShapePlugin)
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
