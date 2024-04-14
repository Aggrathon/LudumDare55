mod camera;
mod projectile;
mod spline;
mod state;
mod tower;
mod unit;
mod utils;

use bevy::prelude::*;
use space_editor::prelude::*;

use camera::CameraPlugin;
use projectile::ProjectilePlugin;
use spline::SplinePlugin;
use state::{Local, StatePlugin};
use tower::TowerPlugin;
use unit::UnitPlugin;

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
        TowerPlugin,
        ProjectilePlugin,
    ));
    #[cfg(feature = "editor")]
    app.add_systems(Startup, space_editor::space_editor_ui::simple_editor_setup);
    #[cfg(not(feature = "editor"))]
    app.add_systems(Startup, spawn_scene)
        .add_systems(PreUpdate, uneditor)
        .init_state::<EditorState>()
        .insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 2048 });
    app.run();
}

#[allow(dead_code)]
fn spawn_scene(mut commands: Commands) {
    commands
        .spawn(PrefabBundle::new("scenes/TestScene.scn.ron"))
        .insert(Local);
}

/// Needed to properly load cameras and lights from space_editor scenes
#[allow(dead_code)]
fn uneditor(
    mut commands: Commands,
    mut cameras: Query<(Entity, &mut Camera), With<PlaymodeCamera>>,
    mut lights: Query<(Entity, &mut DirectionalLight), With<PlaymodeLight>>,
) {
    for (entity, mut camera) in cameras.iter_mut() {
        let bundle = Camera3dBundle::default();
        commands
            .get_entity(entity)
            .unwrap()
            .remove::<PlaymodeCamera>()
            .insert((
                bundle.camera_render_graph,
                bundle.color_grading,
                bundle.dither,
                bundle.exposure,
                bundle.frustum,
                bundle.tonemapping,
                bundle.main_texture_usages,
                bundle.visible_entities,
            ));
        camera.is_active = true;
    }
    for (entity, mut light) in lights.iter_mut() {
        let mut bundle = DirectionalLightBundle::default();
        bundle
            .cascade_shadow_config
            .bounds
            .iter_mut()
            .for_each(|b| *b *= 1.5);
        commands
            .get_entity(entity)
            .unwrap()
            .insert((
                bundle.cascade_shadow_config,
                bundle.cascades,
                bundle.frusta,
                bundle.visible_entities,
            ))
            .remove::<PlaymodeLight>();
        light.shadows_enabled = true;
    }
}
