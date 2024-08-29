mod arc;
mod vertex;

use bevy::prelude::*;

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (vertex::spawn_vertices, arc::spawn_arcs));
    }
}
