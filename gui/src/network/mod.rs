use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_prototype_lyon::prelude::*;

use crate::{shared::*, NetworkWrapper};

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_vertices, spawn_arcs));
    }
}

#[derive(Component)]
struct Vertex(usize);

fn spawn_vertices(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    network: Res<NetworkWrapper>,
    app_settings: Res<AppSettings>,
) {
    for (i, vertex) in network.n.vertices.iter().enumerate() {
        let shape = shapes::Circle {
            radius: 100.,
            ..default()
        };

        let entity = commands
            .spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    spatial: SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(
                            vertex.x,
                            vertex.y,
                            app_settings.vertex_layer,
                        )),
                        ..default()
                    },
                    ..default()
                },
                Stroke::new(app_settings.baseline_color, 15.),
                Fill::color(app_settings.background_color),
                Vertex(i),
            ))
            .id();

        commands.entity(entity).with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    vertex.name.clone(),
                    TextStyle {
                        font_size: 300.0,
                        ..default()
                    },
                ),
                transform: Transform {
                    translation: Vec3::new(0., 0., 1.),
                    scale: Vec3::new(0.2, 0.2, 0.2),
                    ..default()
                },
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
    fixed: bool,
}

impl Arc {
    pub fn arc_point(&self) -> Vec2 {
        let midpoint = self.s_pos.midpoint(self.t_pos);
        let ortho_offshoot = Vec2::new(0., -1.).rotate(self.s_pos - self.t_pos) * 0.2;
        midpoint + ortho_offshoot
    }

    pub fn line_width(&self, min: f32, max: f32) -> f32 {
        let minimum_width = 3.;
        let scaling_factor = 25.;
        let fraction = (self.capacity as f32 - min) / (max - min);
        minimum_width + scaling_factor * (0.5 * (4. * fraction - 2.).tanh() + 0.5)
    }
}

fn spawn_arcs(
    mut commands: Commands,
    network: Res<NetworkWrapper>,
    app_settings: Res<AppSettings>,
) {
    let capacities = network.n.capacities.as_rows();
    let costs = network.n.costs.as_rows();
    let (cap_min, cap_max) = (
        network.n.capacities.min() as f32,
        network.n.capacities.max() as f32,
    );
    for (s, this_vertex) in network.n.vertices.iter().enumerate() {
        for (t, other_vertex) in network.n.vertices.iter().enumerate() {
            let capacity = capacities[s][t];
            let is_fixed = network.n.fixed_arcs.contains(&(s, t));
            if capacity > 0 || is_fixed {
                let arc = Arc {
                    s,
                    t,
                    s_pos: Vec2::new(this_vertex.x, this_vertex.y),
                    t_pos: Vec2::new(other_vertex.x, other_vertex.y),
                    capacity: capacities[s][t],
                    cost: costs[s][t],
                    load: 0,
                    fixed: is_fixed,
                };
                let line_width = arc.line_width(cap_min, cap_max);
                let (color, layer) = if is_fixed {
                    (app_settings.highlight_color, app_settings.arc_fixed_layer)
                } else {
                    (app_settings.baseline_color, app_settings.arc_layer)
                };

                let mut path_builder = PathBuilder::new();
                path_builder.move_to(arc.s_pos);
                path_builder.quadratic_bezier_to(arc.arc_point(), arc.t_pos);
                let path = path_builder.build();

                let entity = commands
                    .spawn((
                        ShapeBundle {
                            path,
                            spatial: SpatialBundle {
                                transform: Transform {
                                    translation: (Vec2::ZERO.extend(layer)),
                                    ..default()
                                },
                                ..default()
                            },
                            ..default()
                        },
                        Stroke::new(color, line_width),
                        arc,
                    ))
                    .id();

                let shape = shapes::RegularPolygon {
                    sides: 3,
                    feature: shapes::RegularPolygonFeature::Radius(line_width.max(6.)),
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
                                        .extend(layer),
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
                        Stroke::new(color, (0.5 * line_width).max(6.)),
                        Fill::color(color),
                    ));
                });
            }
        }
    }
}
