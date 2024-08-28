use bevy::{
    color::Color,
    prelude::{ReflectResource, Resource},
    reflect::Reflect,
};

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct AppSettings {
    pub background_color: Color,
    pub highlight_color: Color,
    pub baseline_color: Color,

    pub vertex_layer: f32,
    pub vertex_selected_layer: f32,
    pub arc_layer: f32,
    pub arc_fixed_layer: f32,
    pub arc_selected_layer: f32,
}
