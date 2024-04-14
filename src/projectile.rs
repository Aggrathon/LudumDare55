use std::time::Duration;

use bevy::prelude::*;
use space_editor::prelude::*;

use crate::state::Gameplay;
use crate::unit::Health;

#[derive(Reflect, Clone, Copy, Default)]
#[reflect(Default)]
pub enum DamageType {
    #[default]
    Physical,
    Magical,
    Explosive,
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
    damage_type: DamageType,
    damage: f32,
    speed: f32,
}

impl Default for HomingProjectile {
    fn default() -> Self {
        Self {
            damage: 10.0,
            damage_type: DamageType::Physical,
            speed: 20.0,
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct RayProjectile {
    damage_type: DamageType,
    damage: f32,
    color: Color,
    duration: Duration,
}

impl Default for RayProjectile {
    fn default() -> Self {
        Self {
            damage: 10.0,
            damage_type: DamageType::Magical,
            color: Color::WHITE,
            duration: Duration::from_secs(1),
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct DumbProjectile {
    damage_type: DamageType,
    damage: f32,
    speed: f32,
}

impl Default for DumbProjectile {
    fn default() -> Self {
        Self {
            damage: 10.0,
            damage_type: DamageType::Explosive,
            speed: 20.0,
        }
    }
}

#[derive(Clone, Copy, Reflect, Default)]
#[reflect(Default)]
pub enum ProjectilePrefab {
    #[default]
    Arrow,
}

impl ProjectilePrefab {
    pub fn path(&self) -> &'static str {
        match self {
            ProjectilePrefab::Arrow => "scenes/Arrow.scn.ron",
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
    time: Res<Time>,
) {
    for (entity, proj, target, mut trans) in q.iter_mut() {
        let delta = target.target - trans.translation;
        let speed = proj.speed * time.delta_seconds();
        let len2 = delta.length_squared();
        if len2 < speed * speed {
            // TODO boom
            println!("boom");
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
        if let Ok((gt, mut health)) = targets.get_mut(target.entity) {
            let delta = gt.translation() - trans.translation;
            let speed = proj.speed * time.delta_seconds();
            let len2 = delta.length_squared();
            if len2 < speed * speed {
                health.0 -= proj.damage;
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
        app.register_type::<DamageType>()
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
