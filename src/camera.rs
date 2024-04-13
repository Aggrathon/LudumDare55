use std::f32::consts::PI;

use bevy::prelude::*;
use space_editor::prelude::*;

use crate::utils::smooth_damp_vec3;

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct CameraTarget {
    speed: f32,
    smooth: f32,
    angle: f32,
    distance: f32,
    #[reflect(ignore)]
    velocity: f32,
}

impl Default for CameraTarget {
    fn default() -> Self {
        Self {
            speed: 10.0,
            smooth: 0.5,
            angle: 60.0,
            distance: 20.0,
            velocity: 0.0,
        }
    }
}

fn target_camera(
    mut targets: Query<(&mut CameraTarget, &Transform)>,
    mut cameras: Query<(&Camera, &mut Transform), Without<CameraTarget>>,
    time: Res<Time>,
) {
    for (mut target, target_trans) in targets.iter_mut() {
        for (camera, mut camera_trans) in cameras.iter_mut() {
            if camera.is_active {
                let rad = target.angle * PI / 180.0;
                let dir = Vec3::new(
                    0.0,
                    target.distance * rad.sin(),
                    target.distance * rad.cos(),
                );
                let (pos, vel) = smooth_damp_vec3(
                    camera_trans.translation,
                    target_trans.translation + dir,
                    target.velocity,
                    target.smooth,
                    target.speed,
                    time.delta_seconds(),
                );
                target.velocity = vel;
                camera_trans.translation = pos;
                camera_trans.look_at(pos - dir, Vec3::Y);
            }
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<CameraTarget>()
            .add_systems(PreUpdate, target_camera.run_if(in_state(EditorState::Game)));
        #[cfg(feature = "editor")]
        app.editor_bundle(
            "Level",
            "Camera Target",
            (
                TransformBundle::default(),
                CameraTarget::default(),
                Name::new("Camera Target"),
            ),
        );
    }
}
