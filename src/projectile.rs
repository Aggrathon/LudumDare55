use std::time::Duration;

use bevy::prelude::*;
use space_editor::prelude::*;

use crate::level::Gameplay;
use crate::unit::Health;

#[derive(Reflect, Clone, Copy, PartialEq)]
#[reflect(Default)]
pub enum Damage {
    Physical(f32),
    Magical(f32),
    Explosive(f32, f32),
}

impl Default for Damage {
    fn default() -> Self {
        Damage::Physical(20.0)
    }
}

#[derive(Component, Clone, Copy)]
pub struct DespawnTimer(Duration);

#[derive(Component, Clone)]
pub struct ProjectileTarget {
    entity: Entity,
    target: Vec3,
    pos: Vec3,
}

impl ProjectileTarget {
    pub fn new(entity: Entity, target: Vec3, pos: Vec3) -> Self {
        Self {
            entity,
            target,
            pos,
        }
    }
}

#[allow(clippy::type_complexity)]
fn setup_projectile(
    mut commands: Commands,
    q: Query<
        (Entity, &ProjectileTarget, &Children),
        (
            Without<HomingProjectile>,
            Without<DumbProjectile>,
            Without<RayProjectile>,
        ),
    >,
) {
    for (e, sp, ch) in q.iter() {
        for c in ch.iter() {
            commands
                .get_entity(e)
                .unwrap()
                .remove::<ProjectileTarget>()
                .insert(Transform::default());
            commands.get_entity(*c).unwrap().insert((
                sp.clone(),
                Transform {
                    translation: sp.pos,
                    ..default()
                },
            ));
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct HomingProjectile {
    damage: Damage,
    speed: f32,
}

impl Default for HomingProjectile {
    fn default() -> Self {
        Self {
            damage: Damage::Physical(10.0),
            speed: 20.0,
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct RayProjectile {
    damage: Damage,
    color: Color,
    duration: Duration,
}

impl Default for RayProjectile {
    fn default() -> Self {
        Self {
            damage: Damage::Magical(10.0),
            color: Color::WHITE,
            duration: Duration::from_secs(1),
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct DumbProjectile {
    damage: Damage,
    speed: f32,
}

impl Default for DumbProjectile {
    fn default() -> Self {
        Self {
            damage: Damage::Explosive(10.0, 4.0),
            speed: 20.0,
        }
    }
}

#[derive(Clone, Copy, Reflect, Default)]
#[reflect(Default)]
pub enum ProjectilePrefab {
    #[default]
    Arrow,
    Bomb,
}

impl ProjectilePrefab {
    pub const fn path(&self) -> &'static str {
        match self {
            ProjectilePrefab::Arrow => "scenes/Arrow.scn.ron",
            ProjectilePrefab::Bomb => "scenes/Bomb.scn.ron",
        }
    }
}

fn despawn_timer(mut commands: Commands, q: Query<(Entity, &DespawnTimer)>, time: Res<Time>) {
    let time = time.elapsed();
    for (e, t) in q.iter() {
        if t.0 < time {
            commands.get_entity(e).unwrap().despawn_recursive();
        }
    }
}

fn shoot_dumb(
    mut commands: Commands,
    mut q: Query<(Entity, &DumbProjectile, &ProjectileTarget, &mut Transform)>,
    mut targets: Query<(&GlobalTransform, &mut Health)>,
    time: Res<Time>,
) {
    for (entity, proj, target, mut trans) in q.iter_mut() {
        let delta = target.target + Vec3::Y * 0.3 - trans.translation;
        let speed = proj.speed * time.delta_seconds();
        let len2 = delta.length_squared();
        if len2 < speed * speed {
            deal_damage(trans.translation, proj.damage, target.entity, &mut targets);
            commands.get_entity(entity).unwrap().despawn_recursive();
        } else {
            trans.translation += delta * (speed / len2.sqrt());
            trans.look_to(delta, Vec3::Y);
        }
    }
}

fn shoot_homing(
    mut commands: Commands,
    mut q: Query<(Entity, &HomingProjectile, &ProjectileTarget, &mut Transform)>,
    mut targets: Query<(&GlobalTransform, &mut Health)>,
    time: Res<Time>,
) {
    for (entity, proj, target, mut trans) in q.iter_mut() {
        if let Ok((gt, _)) = targets.get(target.entity) {
            let delta = gt.translation() + Vec3::Y * 0.3 - trans.translation;
            let speed = proj.speed * time.delta_seconds();
            let len2 = delta.length_squared();
            if len2 < speed * speed {
                deal_damage(trans.translation, proj.damage, target.entity, &mut targets);
                commands.get_entity(entity).unwrap().despawn_recursive();
            } else {
                trans.translation += delta * (speed / len2.sqrt());
                trans.look_to(delta, Vec3::Y);
            }
        } else {
            commands.get_entity(entity).unwrap().despawn_recursive();
        }
    }
}

fn deal_damage(
    pos: Vec3,
    damage: Damage,
    target: Entity,
    targets: &mut Query<(&GlobalTransform, &mut Health)>,
) {
    match damage {
        Damage::Physical(d) => {
            if let Ok((_, mut health)) = targets.get_mut(target) {
                health.0 -= d;
            }
        }
        Damage::Magical(d) => {
            if let Ok((_, mut health)) = targets.get_mut(target) {
                health.0 -= d;
            }
        }
        Damage::Explosive(d, r) => {
            let r2 = r * r;
            for (gt, mut health) in targets.iter_mut() {
                if pos.distance_squared(gt.translation()) < r2 {
                    health.0 -= d;
                }
                // TODO FX
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn setup_ray(
    mut commands: Commands,
    q: Query<(Entity, &RayProjectile), Without<DespawnTimer>>,
    time: Res<Time>,
) {
    for (entity, ray) in q.iter() {
        commands
            .get_entity(entity)
            .unwrap()
            .insert(DespawnTimer(time.elapsed() + ray.duration));
    }
}

fn shoot_ray(q: Query<(&RayProjectile, &ProjectileTarget, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (proj, target, trans) in q.iter() {
        gizmos.line(trans.translation(), target.target, proj.color);
        // TODO zap
        println!("zap");
    }
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Damage>()
            .register_type::<ProjectilePrefab>()
            .editor_registry::<HomingProjectile>()
            .editor_registry::<RayProjectile>()
            .editor_registry::<DumbProjectile>()
            .add_systems(PreUpdate, (setup_projectile, setup_ray).in_set(Gameplay))
            .add_systems(
                Update,
                (shoot_dumb, shoot_homing, shoot_ray, despawn_timer).in_set(Gameplay),
            );
        #[cfg(feature = "editor")]
        app.editor_bundle(
            "Prefab",
            "Homing Projectile",
            (
                SpatialBundle::default(),
                HomingProjectile::default(),
                Name::new("HomingProjectile"),
            ),
        );
        #[cfg(feature = "editor")]
        app.editor_bundle(
            "Prefab",
            "Ray Projectile",
            (
                SpatialBundle::default(),
                RayProjectile::default(),
                Name::new("RayProjectile"),
            ),
        );
        #[cfg(feature = "editor")]
        app.editor_bundle(
            "Prefab",
            "Dumb Projectile",
            (
                SpatialBundle::default(),
                DumbProjectile::default(),
                Name::new("DumbProjectile"),
            ),
        );
    }
}
