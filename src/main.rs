mod camera;
mod spline;
mod state;
mod unit;
mod utils;

use bevy::prelude::*;
use space_editor::prelude::*;

use crate::camera::CameraPlugin;
use crate::spline::SplinePlugin;
use crate::state::{Local, StatePlugin};
use crate::unit::UnitPlugin;

fn main() {
    let mut app = App::default();
    app.add_plugins((
        DefaultPlugins,
        #[cfg(feature = "editor")]
        SpaceEditorPlugin,
        #[cfg(not(feature = "editor"))]
        PrefabPlugin,
        SplinePlugin,
        UnitPlugin,
        CameraPlugin,
        StatePlugin,
    ));
    #[cfg(feature = "editor")]
    app.add_systems(Startup, space_editor::space_editor_ui::simple_editor_setup);
    #[cfg(not(feature = "editor"))]
    app.add_systems(Startup, spawn_scene)
        .init_state::<EditorState>();
    app.run();
}

#[allow(dead_code)]
fn spawn_scene(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands
        .spawn(PrefabBundle::new("scenes/TestScene.scn.ron"))
        .insert(Local);
}
