mod arc;
mod vertex;

use arc::Arc;
use bevy::prelude::*;
use vertex::Vertex;

use crate::NetworkWrapper;

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (vertex::spawn_vertices, arc::spawn_arcs))
            .add_systems(Update, (vertex::create_vertex, save_network));
    }
}

pub fn save_network(
    mut network: ResMut<NetworkWrapper>,
    query_vertex: Query<(&Vertex, &mut Transform)>,
    query_arc: Query<&Arc>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !keys.just_pressed(KeyCode::KeyS) {
        return;
    }
    for (vertex, transform) in query_vertex.iter() {
        network.n.vertices[vertex.0].x = transform.translation.x;
        network.n.vertices[vertex.0].y = transform.translation.y;
    }

    network.n.fixed_arcs = query_arc
        .iter()
        .filter(|arc| arc.fixed)
        .map(|arc| (arc.s, arc.t))
        .collect();
    network.n.serialize("gui_output.json").unwrap();
}
