mod audio;
mod camera;
mod fx;
mod level;
mod projectile;
mod spline;
mod tower;
mod ui;
mod unit;
mod utils;

use bevy::prelude::*;
use space_editor::prelude::*;

use audio::AudioPlugin;
use camera::CameraPlugin;
use fx::FxPlugin;
use level::{Level, LevelPlugin};
use projectile::ProjectilePlugin;
use spline::SplinePlugin;
use tower::TowerPlugin;
use ui::UiPlugin;
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
        LevelPlugin,
        TowerPlugin,
        ProjectilePlugin,
        AudioPlugin,
        UiPlugin,
        FxPlugin,
    ));
    #[cfg(feature = "editor")]
    app.add_systems(Startup, space_editor::space_editor_ui::simple_editor_setup);
    #[cfg(not(feature = "editor"))]
    app.add_systems(PreUpdate, noeditor)
        .init_state::<EditorState>()
        .add_systems(Startup, |mut level: ResMut<NextState<Level>>| {
            level.set(Level::MainMenu)
        })
        .insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 2048 });
    app.run();
}

/// Needed to properly load cameras and lights from space_editor scenes
#[allow(dead_code)]
fn noeditor(
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
            .for_each(|b| *b *= 2.0);
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
