use bevy::prelude::*;
use bevy_mod_picking::events::{Drag, DragEnd, DragEnter, DragLeave, DragStart, Pointer};
use bevy_mod_picking::focus::PickingInteraction;
use bevy_mod_picking::prelude::{Listener, On, Pickable, PointerButton};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::prelude::*;

use crate::camera::{BackgroundMarker, WorldCoords, Zoom};
use crate::network::arc::*;
use crate::{shared::*, NetworkWrapper};

#[derive(Component)]
pub struct Vertex(pub usize);

impl Vertex {
    pub fn spawn(
        vertex_number: usize,
        vertex: &robmcf_greedy::Vertex,
        commands: &mut Commands,
        app_settings: &Res<AppSettings>,
    ) {
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
                            app_settings.vertex_layer + vertex_number as f32 * 0.001,
                        )),
                        ..default()
                    },
                    ..default()
                },
                Stroke::new(app_settings.baseline_color, 15.),
                Fill::color(app_settings.background_color),
                Vertex(vertex_number),
                PickableBundle::default(),
                On::<Pointer<Drag>>::run(drag_vertex),
                On::<Pointer<DragStart>>::run(begin_arc_creation),
                On::<Pointer<DragEnter>>::run(snap_arc_to_vertex),
                On::<Pointer<DragLeave>>::run(unsnap_arc_from_vertex),
                On::<Pointer<DragEnd>>::run(finalize_arc_creation),
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

pub fn spawn_vertices(
    mut commands: Commands,
    network: Res<NetworkWrapper>,
    app_settings: Res<AppSettings>,
) {
    for (i, vertex) in network.n.vertices.iter().enumerate() {
        Vertex::spawn(i, vertex, &mut commands, &app_settings);
    }
}

fn drag_vertex(
    drag: Listener<Pointer<Drag>>,
    mut query: Query<(&Vertex, &mut Transform)>,
    mut query_arcs: Query<(&mut Arc, &mut Path, &Children), Without<PartialArc>>,
    mut query_partial_arc: Query<
        (&mut Arc, &mut Path, &Children),
        (With<PartialArc>, Without<Vertex>),
    >,
    mut query_arrows: Query<&mut Transform, (With<Arrow>, Without<Vertex>)>,
    zoom_query: Query<&Zoom>,
    world_coords: Res<WorldCoords>,
) {
    let Ok((vertex, mut transform)) = query.get_mut(drag.target) else {
        return;
    };
    let Ok(zoom) = zoom_query.get_single() else {
        return;
    };

    let transform_x = transform.translation.x + drag.delta.x * zoom.target;
    let transform_y = transform.translation.y - drag.delta.y * zoom.target;

    if drag.button == PointerButton::Primary {
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

    if drag.button == PointerButton::Secondary {
        let Ok((mut arc, mut path, children)) = query_partial_arc.get_single_mut() else {
            return;
        };

        if arc.s == usize::MAX {
            arc.s = vertex.0;
            arc.s_pos.x = transform_x;
            arc.s_pos.y = transform_y;
            arc.t_pos.x = transform_x;
            arc.t_pos.y = transform_y;
        }
        if arc.t == usize::MAX {
            arc.t_pos = world_coords.0;
        }

        *path = arc.get_path();

        let mut arrow = query_arrows.get_mut(children[0]).unwrap();
        let arrow_translation = arc.get_arrow_translation();
        arrow.translation.x = arrow_translation.x;
        arrow.translation.y = arrow_translation.y;
        arrow.rotation = arc.get_arrow_rotation();
    }
}

pub fn create_vertex(
    mouse_input: Res<ButtonInput<MouseButton>>,
    input_query: Query<
        Option<&PickingInteraction>,
        (With<BackgroundMarker>, Changed<PickingInteraction>),
    >,
    world_coords: Res<WorldCoords>,
    mut network: ResMut<NetworkWrapper>,
    app_settings: Res<AppSettings>,
    mut commands: Commands,
) {
    if !mouse_input.just_pressed(MouseButton::Right) {
        return;
    }
    let Ok(Some(PickingInteraction::Pressed)) = input_query.get_single() else {
        return;
    };

    let vertex = robmcf_greedy::Vertex {
        name: format!("v{}", network.num_vertices),
        x: world_coords.0.x,
        y: world_coords.0.y,
    };
    Vertex::spawn(network.num_vertices, &vertex, &mut commands, &app_settings);
    network.num_vertices += 1;

    let row = vec![0; network.num_vertices - 1];
    let column = vec![0; network.num_vertices];

    network.n.vertices.push(vertex);
    network.n.capacities.extend(&row, &column);
    network.n.costs.extend(&row, &column);
    for balance in &mut network.n.balances {
        balance.extend(&row, &column);
    }
}
