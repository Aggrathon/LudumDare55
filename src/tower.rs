use std::time::Duration;

use bevy::prelude::*;
use space_editor::prelude::*;

use crate::level::{Gameplay, LevelLocal};
use crate::projectile::{ProjectilePrefab, ProjectileTarget};
use crate::spline::FollowCurve;
use crate::unit::Unit;

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct Targetter {
    range: f32,
}

impl Default for Targetter {
    fn default() -> Self {
        Self { range: 10.0 }
    }
}

#[derive(Component, Reflect, Copy, Clone, Default)]
#[reflect(Component, Default)]
pub struct LookAtTarget;

#[derive(Component, Clone, Copy)]
pub struct Target(pub Entity);

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct Tower {
    projectile: ProjectilePrefab,
    cooldown: Duration,
    height: f32,
    #[reflect(ignore)]
    next: Duration,
}

impl Default for Tower {
    fn default() -> Self {
        Self {
            cooldown: Duration::from_secs(1),
            height: 1.0,
            projectile: ProjectilePrefab::Arrow,
            next: Duration::ZERO,
        }
    }
}

fn find_target(
    mut commands: Commands,
    q: Query<(Entity, &Targetter, &GlobalTransform), Without<Target>>,
    units: Query<(Entity, &FollowCurve, &GlobalTransform), With<Unit>>,
) {
    for (entity, targetter, gt) in q.iter() {
        let pos = gt.translation();
        let mut furthest = f32::MIN;
        for (target, fc, gt) in units.iter() {
            if pos.distance_squared(gt.translation()) < targetter.range * targetter.range
                && fc.distance() > furthest
            {
                commands.get_entity(entity).unwrap().insert(Target(target));
                furthest = fc.distance();
            }
        }
    }
}
fn look_at_target(
    mut q: Query<(&Target, &GlobalTransform, &mut Transform), With<LookAtTarget>>,
    units: Query<&GlobalTransform, With<Unit>>,
) {
    for (target, gt, mut trans) in q.iter_mut() {
        if let Ok(gt2) = units.get(target.0) {
            let mut dir = gt2.translation() - gt.translation();
            dir.y = 0.0;
            trans.look_to(dir, Vec3::Y);
        }
    }
}

fn shoot(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Tower, &Targetter, &Target, &GlobalTransform)>,
    units: Query<(Entity, &GlobalTransform), With<Unit>>,
    time: Res<Time>,
) {
    let time = time.elapsed();
    for (entity, mut tower, targetter, target, gt) in q.iter_mut() {
        if tower.next < time {
            if let Ok((unit, gt2)) = units.get(target.0) {
                if gt.translation().distance_squared(gt2.translation())
                    < targetter.range * targetter.range
                {
                    tower.next = time + tower.cooldown;
                    commands
                        .spawn(PrefabBundle::new(tower.projectile.path()))
                        .insert((
                            LevelLocal,
                            ProjectileTarget::new(
                                unit,
                                gt2.translation(),
                                gt.translation() + gt.up() * tower.height,
                            ),
                        ));
                } else {
                    commands.get_entity(entity).unwrap().remove::<Target>();
                }
            } else {
                commands.get_entity(entity).unwrap().remove::<Target>();
            }
        }
    }
}

#[allow(unused)]
fn debug_gizmos(q: Query<(&Targetter, &Tower, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (targetter, tower, gt) in q.iter() {
        gizmos.circle(
            gt.translation() + gt.up() * 0.1,
            Direction3d::Y,
            targetter.range,
            Color::RED,
        );
        gizmos.arrow(
            gt.translation(),
            gt.translation() + gt.up() * tower.height,
            Color::RED,
        );
    }
}

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<Targetter>()
            .editor_registry::<LookAtTarget>()
            .editor_registry::<Tower>()
            .add_systems(
                Update,
                (find_target, look_at_target, shoot).in_set(Gameplay),
            );
        #[cfg(feature = "editor")]
        app.add_systems(Update, debug_gizmos.run_if(in_state(EditorState::Editor)))
            .editor_bundle(
                "Prefab",
                "Tower",
                (
                    SpatialBundle::default(),
                    Targetter::default(),
                    Tower::default(),
                    Name::new("Tower"),
                ),
            );
    }
}
