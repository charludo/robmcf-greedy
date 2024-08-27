use robmcf_greedy::Network;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_vertices);
        // .add_systems(Update, ());
    }
}

#[derive(Component)]
struct Vertex {}

#[derive(Bundle)]
struct VertexBundle {
    vertex: Vertex,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}

fn spawn_vertices(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let n = Network::from_random(
        20,       // num_vertices,
        0.6,      // connectedness,
        0.3,      // supply_density,
        2,        // num_scenarios,
        (3, 8),   // range_supply,
        (15, 40), // range_capacity,
        (4, 8),   // range_cost,
        5,        // num_fixed_arcs,
    );

    for vertex in n.vertices {
        let entity = commands
            .spawn(VertexBundle {
                vertex: Vertex {},
                mesh: MaterialMesh2dBundle {
                    mesh: meshes.add(Circle::new(100.)).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation(Vec3::new(vertex.x, vertex.y, 0.)),
                    ..default()
                },
            })
            .id();

        commands.entity(entity).with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(Circle::new(90.)).into(),
                material: materials.add(ColorMaterial::from(Color::srgb(0.16, 0.16, 0.18))),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.5)),
                ..default()
            });
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    vertex.name,
                    TextStyle {
                        font_size: 50.0,
                        ..default()
                    },
                ),
                transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                ..default()
            });
        });
    }
}
