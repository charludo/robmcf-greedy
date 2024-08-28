use bevy::prelude::*;
use bevy_mod_picking::events::{Drag, Pointer};
use bevy_mod_picking::prelude::{Listener, On};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::prelude::*;

use crate::camera::Zoom;
use crate::{shared::*, NetworkWrapper};

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_vertices, spawn_arcs));
    }
}

#[derive(Component)]
struct Vertex(usize);

fn drag_vertex(
    drag: Listener<Pointer<Drag>>,
    mut query: Query<(Entity, &Vertex, &mut Transform)>,
    mut query_arcs: Query<(&mut Arc, &mut Path, &Children), Without<Vertex>>,
    mut query_arrows: Query<&mut Transform, (With<Arrow>, Without<Vertex>)>,
    zoom_query: Query<&Zoom>,
) {
    let Ok((_, vertex, mut transform)) = query.get_mut(drag.target) else {
        return;
    };
    let zoom = zoom_query.get_single().unwrap();
    let transform_x = transform.translation.x + drag.delta.x * zoom.target;
    let transform_y = transform.translation.y - drag.delta.y * zoom.target;

    transform.translation.x = transform_x;
    transform.translation.y = transform_y;

    for (mut arc, mut path, children) in query_arcs
        .iter_mut()
        .filter(|(a, _, _)| a.s == vertex.0 || a.t == vertex.0)
    {
        if arc.s == vertex.0 {
            arc.s_pos.x = transform_x;
            arc.s_pos.y = transform_y;
        } else {
            arc.t_pos.x = transform_x;
            arc.t_pos.y = transform_y;
        }
        *path = arc.get_path();

        let mut arrow = query_arrows.get_mut(children[0]).unwrap();
        let arrow_translation = arc.get_arrow_translation();
        arrow.translation.x = arrow_translation.x;
        arrow.translation.y = arrow_translation.y;
        arrow.rotation = arc.get_arrow_rotation();
    }
}

fn spawn_vertices(
    mut commands: Commands,
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
                            app_settings.vertex_layer + i as f32 * 0.001,
                        )),
                        ..default()
                    },
                    ..default()
                },
                Stroke::new(app_settings.baseline_color, 15.),
                Fill::color(app_settings.background_color),
                Vertex(i),
                PickableBundle::default(),
                On::<Pointer<Drag>>::run(drag_vertex),
            ))
            .id();

        commands.entity(entity).with_children(|parent| {
            parent.spawn((Text2dBundle {
                text: Text::from_section(
                    vertex.name.clone(),
                    TextStyle {
                        font_size: 300.0,
                        ..default()
                    },
                ),
                transform: Transform {
                    translation: Vec3::new(0., 0., 0.001),
                    scale: Vec3::new(0.2, 0.2, 0.2),
                    ..default()
                },
                ..default()
            },));
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

#[derive(Component)]
struct Arrow;

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

    pub fn get_path(&self) -> Path {
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(self.s_pos);
        path_builder.quadratic_bezier_to(self.arc_point(), self.t_pos);
        let path = path_builder.build();
        path
    }

    pub fn get_arrow_translation(&self) -> Vec2 {
        self.s_pos.midpoint(self.t_pos) + Vec2::new(0., -1.).rotate(self.s_pos - self.t_pos) * 0.1
    }

    pub fn get_arrow_rotation(&self) -> Quat {
        Quat::from_axis_angle(
            Vec3::new(0., 0., 1.),
            Vec2::new(0., -1.).angle_between(self.s_pos - self.t_pos),
        )
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

                let entity = commands
                    .spawn((
                        ShapeBundle {
                            path: arc.get_path(),
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
                        PickableBundle::default(),
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
                                    translation: arc.get_arrow_translation().extend(layer),
                                    rotation: arc.get_arrow_rotation(),
                                    ..default()
                                },
                                ..default()
                            },
                            ..default()
                        },
                        Stroke::new(color, (0.5 * line_width).max(6.)),
                        Fill::color(color),
                        Arrow,
                    ));
                });
            }
        }
    }
}
