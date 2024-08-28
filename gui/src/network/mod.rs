use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_prototype_lyon::prelude::*;

use crate::NetworkWrapper;

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_vertices, draw_arcs));
        // .add_systems(Update, draw_arcs);
    }
}

#[derive(Component)]
struct Vertex(usize);

#[derive(Bundle)]
struct VertexBundle {
    vertex: Vertex,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}

fn spawn_vertices(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    network: Res<NetworkWrapper>,
) {
    for (i, vertex) in network.n.vertices.iter().enumerate() {
        let entity = commands
            .spawn(VertexBundle {
                vertex: Vertex(i),
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
                    vertex.name.clone(),
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

#[derive(Component, Copy, Clone)]
struct Arc {
    s: usize,
    t: usize,
    s_pos: Vec2,
    t_pos: Vec2,
    capacity: usize,
    cost: usize,
    load: usize,
}

impl Arc {
    pub fn arc_point(&self) -> Vec2 {
        let midpoint = self.s_pos.midpoint(self.t_pos);
        let ortho_offshoot = Vec2::new(0., -1.).rotate(self.s_pos - self.t_pos) * 0.2;
        midpoint + ortho_offshoot
    }

    pub fn line_width(&self, min: f32, max: f32) -> f32 {
        let minimum_width = 4.;
        let scaling_factor = 40.;
        let fraction = (self.capacity as f32 - min) / (max - min);
        minimum_width + scaling_factor * 0.5 * (4. * fraction - 2.).tanh() + 0.5
    }
}

fn draw_arcs(mut commands: Commands, network: Res<NetworkWrapper>) {
    let capacities = network.n.capacities.as_rows();
    let costs = network.n.costs.as_rows();
    let (cap_min, cap_max) = (
        network.n.capacities.min() as f32,
        network.n.capacities.max() as f32,
    );
    for s in 0..network.n.vertices.len() {
        let this_vertex = &network.n.vertices[s];
        for (t, capacity) in capacities[s].iter().enumerate() {
            if *capacity > 0 {
                let other_vertex = &network.n.vertices[t];
                let arc = Arc {
                    s,
                    t,
                    s_pos: Vec2::new(this_vertex.x, this_vertex.y),
                    t_pos: Vec2::new(other_vertex.x, other_vertex.y),
                    capacity: capacities[s][t],
                    cost: costs[s][t],
                    load: 0,
                };
                let line_width = arc.line_width(cap_min, cap_max);

                let mut path_builder = PathBuilder::new();
                path_builder.move_to(arc.s_pos);
                path_builder.quadratic_bezier_to(arc.arc_point(), arc.t_pos);
                let path = path_builder.build();

                let entity = commands
                    .spawn((
                        ShapeBundle { path, ..default() },
                        Stroke::new(Color::WHITE, line_width),
                        arc,
                    ))
                    .id();

                let shape = shapes::RegularPolygon {
                    sides: 3,
                    feature: shapes::RegularPolygonFeature::Radius(line_width.max(4.0)),
                    ..shapes::RegularPolygon::default()
                };
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&shape),
                            spatial: SpatialBundle {
                                transform: Transform {
                                    translation: (arc.s_pos.midpoint(arc.t_pos)
                                        + Vec2::new(0., -1.).rotate(arc.s_pos - arc.t_pos) * 0.1)
                                        .extend(0.),
                                    rotation: Quat::from_axis_angle(
                                        Vec3::new(0., 0., 1.),
                                        Vec2::new(0., -1.).angle_between(arc.s_pos - arc.t_pos),
                                    ),
                                    ..default()
                                },
                                ..default()
                            },
                            ..default()
                        },
                        Stroke::new(Color::WHITE, 10.0),
                        Fill::color(Color::WHITE),
                    ));
                });
            }
        }
    }
}
