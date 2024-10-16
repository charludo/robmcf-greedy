use bevy::prelude::*;
use bevy_mod_picking::events::{Click, DragEnd, DragEnter, DragLeave, DragStart, Pointer};
use bevy_mod_picking::prelude::{Listener, On, Pickable, PointerButton};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::prelude::*;
use rand::Rng;
use robmcf_greedy::Matrix;

use crate::network::vertex::*;
use crate::{shared::*, NetworkWrapper};

#[derive(Component, Copy, Clone)]
pub struct Arc {
    pub s: usize,
    pub t: usize,
    pub s_pos: Vec2,
    pub t_pos: Vec2,
    pub capacity: usize,
    pub cost: usize,
    pub load: usize,
    pub fixed: bool,

    pub color: Color,
    pub layer: f32,
    pub line_width: f32,
}

#[derive(Component)]
pub struct Arrow;

#[derive(Component)]
pub struct PartialArc;

impl Arc {
    pub fn arc_point(&self) -> Vec2 {
        let midpoint = self.s_pos.midpoint(self.t_pos);
        let ortho_offshoot = Vec2::new(0., -1.).rotate(self.s_pos - self.t_pos) * 0.2;
        midpoint + ortho_offshoot
    }

    pub fn line_width(capacity: usize, max: f32) -> f32 {
        let minimum_width = 3.;
        let scaling_factor = 25.;
        let fraction = (capacity as f32) / max;
        minimum_width + scaling_factor * (0.5 * (4. * fraction - 2.).tanh() + 0.5)
    }

    pub fn get_path(&self) -> Path {
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(self.s_pos);
        path_builder.quadratic_bezier_to(self.arc_point(), self.t_pos);
        path_builder.build()
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

    #[allow(clippy::complexity)]
    pub fn spawn_arc(
        &self,
    ) -> (
        (ShapeBundle, Stroke, Arc, PickableBundle, On<Pointer<Click>>),
        (ShapeBundle, Stroke, Fill, Arrow, PickableBundle),
    ) {
        let mut rng = rand::thread_rng();
        let rand_offset = rng.gen_range(0.01..0.5);

        let line = (
            ShapeBundle {
                path: self.get_path(),
                spatial: SpatialBundle {
                    transform: Transform::from_translation(
                        Vec2::ZERO.extend(self.layer + rand_offset),
                    ),
                    ..default()
                },
                ..default()
            },
            Stroke::new(self.color, self.line_width),
            *self,
            PickableBundle::default(),
            On::<Pointer<Click>>::run(toggle_arc_fixed),
        );

        let shape = shapes::RegularPolygon {
            sides: 3,
            feature: shapes::RegularPolygonFeature::Radius(self.line_width.max(6.)),
            ..shapes::RegularPolygon::default()
        };

        let arrow = (
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                spatial: SpatialBundle {
                    transform: Transform {
                        translation: self
                            .get_arrow_translation()
                            .extend(self.layer + rand_offset + 0.001),
                        rotation: self.get_arrow_rotation(),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            Stroke::new(self.color, (0.5 * self.line_width).max(6.)),
            Fill::color(self.color),
            Arrow,
            PickableBundle {
                pickable: Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
                ..default()
            },
        );

        (line, arrow)
    }
}

pub fn spawn_arcs(
    mut commands: Commands,
    network: Res<NetworkWrapper>,
    app_settings: Res<AppSettings>,
) {
    let capacities = network.n.capacities.as_rows();
    let costs = network.n.costs.as_rows();
    let cap_max = network.n.capacities.max() as f32;
    let (flows, flow_max): (Matrix<usize>, usize) = match &network.n.baseline {
        Some(baseline) => {
            let mut flows =
                Matrix::filled_with(0, network.n.vertices.len(), network.n.vertices.len());
            for solution in baseline.iter() {
                flows = flows.add(&solution.arc_loads);
            }
            (flows.clone(), *flows.elements().max().unwrap_or(&0))
        }
        None => (Matrix::empty(), 0),
    };
    for (s, this_vertex) in network.n.vertices.iter().enumerate() {
        for (t, other_vertex) in network.n.vertices.iter().enumerate() {
            let capacity = capacities[s][t];
            let is_fixed = network.n.fixed_arcs.contains(&(s, t));
            if capacity > 0 {
                let (color, layer) = if is_fixed {
                    (app_settings.highlight_color, app_settings.arc_fixed_layer)
                } else {
                    match &network.n.baseline {
                        Some(_baseline) => {
                            let scaled = 0.9 * (*flows.get(s, t) as f32 / flow_max as f32);
                            let mut color = app_settings.baseline_color.to_owned();
                            color.set_alpha(0.1 + scaled);
                            (color, app_settings.arc_fixed_layer)
                        }
                        None => (app_settings.baseline_color, app_settings.arc_layer),
                    }
                };
                let arc = Arc {
                    s,
                    t,
                    s_pos: Vec2::new(this_vertex.x, this_vertex.y),
                    t_pos: Vec2::new(other_vertex.x, other_vertex.y),
                    capacity: capacities[s][t],
                    cost: costs[s][t],
                    load: 0,
                    fixed: is_fixed,

                    color,
                    layer,
                    line_width: Arc::line_width(capacity, cap_max),
                };

                let (line, arrow) = arc.spawn_arc();
                let line_entity = commands.spawn(line).id();
                commands.entity(line_entity).with_children(|parent| {
                    parent.spawn(arrow);
                });
            }
        }
    }
}

pub fn begin_arc_creation(
    drag: Listener<Pointer<DragStart>>,
    network: Res<NetworkWrapper>,
    app_settings: Res<AppSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if drag.button != PointerButton::Secondary {
        return;
    }

    let is_fixed = keys.pressed(KeyCode::ControlLeft);
    let (color, layer) = if is_fixed {
        (app_settings.highlight_color, app_settings.arc_fixed_layer)
    } else {
        (app_settings.baseline_color, app_settings.arc_selected_layer)
    };

    let arc = Arc {
        s: usize::MAX,
        t: usize::MAX,
        s_pos: Vec2::ZERO,
        t_pos: Vec2::ZERO,
        capacity: 1,
        cost: 0,
        load: 0,
        fixed: is_fixed,

        color,
        layer,
        line_width: Arc::line_width(1, network.n.capacities.max() as f32),
    };

    let (line, arrow) = arc.spawn_arc();
    let line_entity = commands.spawn(line).id();
    commands
        .entity(line_entity)
        .insert(PartialArc)
        .with_children(|parent| {
            parent.spawn(arrow);
        });
}

pub fn snap_arc_to_vertex(
    drag: Listener<Pointer<DragEnter>>,
    mut query_arc: Query<&mut Arc, With<PartialArc>>,
    query_vertex: Query<(&Vertex, &Transform)>,
) {
    if drag.button != PointerButton::Secondary {
        return;
    }

    let Ok(mut arc) = query_arc.get_single_mut() else {
        return;
    };

    let Ok((vertex, transform)) = query_vertex.get(drag.target) else {
        return;
    };

    if arc.s != vertex.0 {
        arc.t = vertex.0;
        arc.t_pos = transform.translation.truncate();
    }
}

pub fn unsnap_arc_from_vertex(
    drag: Listener<Pointer<DragLeave>>,
    mut query_arc: Query<&mut Arc, With<PartialArc>>,
    query_vertex: Query<&Vertex>,
) {
    if drag.button != PointerButton::Secondary {
        return;
    }

    let Ok(mut arc) = query_arc.get_single_mut() else {
        return;
    };

    let Ok(vertex) = query_vertex.get(drag.target) else {
        return;
    };

    if arc.s != vertex.0 {
        arc.t = usize::MAX;
    }
}

pub fn finalize_arc_creation(
    drag: Listener<Pointer<DragEnd>>,
    query_arc: Query<(Entity, &Arc), With<PartialArc>>,
    mut network: ResMut<NetworkWrapper>,
    mut commands: Commands,
) {
    if drag.button != PointerButton::Secondary {
        return;
    }

    let Ok((entity, arc)) = query_arc.get_single() else {
        return;
    };

    if arc.s == usize::MAX || arc.t == usize::MAX || *network.n.capacities.get(arc.s, arc.t) > 0 {
        commands.entity(entity).despawn_recursive();
        return;
    }

    commands.entity(entity).remove::<PartialArc>();
    network.n.capacities.set(arc.s, arc.t, arc.capacity);
    network.n.costs.set(arc.s, arc.t, arc.cost);
}

pub fn toggle_arc_fixed(
    click: Listener<Pointer<Click>>,
    mut query_arc: Query<(&mut Arc, &mut Stroke)>,
    app_settings: Res<AppSettings>,
) {
    if click.button != PointerButton::Secondary {
        return;
    }

    let Ok((mut arc, mut stroke)) = query_arc.get_mut(click.target) else {
        return;
    };

    arc.fixed = !arc.fixed;
    if arc.fixed {
        arc.color = app_settings.highlight_color;
        println!("Not fixed -> fixed");
    } else {
        arc.color = app_settings.baseline_color;
        println!("Fixed -> not fixed");
    }
    stroke.color = arc.color;
}
