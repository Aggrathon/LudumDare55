use std::time::Duration;

use bevy::prelude::*;
use space_editor::prelude::*;

use crate::spline::{Curve, FollowCurve};
use crate::state::{Gameplay, Local};
use crate::utils::get_random_from_iter;

#[derive(Clone, Copy, Reflect, Default)]
#[reflect(Default)]
pub enum UnitPrefab {
    #[default]
    PlaceHolder,
}

impl UnitPrefab {
    pub fn path(&self) -> &'static str {
        match self {
            UnitPrefab::PlaceHolder => "scenes/PlaceholderUnit.scn.ron",
        }
    }
}

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct Unit {
    speed: f32,
}

impl Default for Unit {
    fn default() -> Self {
        Self { speed: 2.0 }
    }
}

pub fn instantiate_unit(
    mut commands: Commands,
    units: Query<(Entity, &Unit), Without<FollowCurve>>,
    curves: Query<Entity, With<Curve>>,
) {
    for (entity, unit) in units.iter() {
        match get_random_from_iter(|| curves.iter()) {
            None => continue,
            Some(curve) => commands
                .get_entity(entity)
                .unwrap()
                .insert(FollowCurve::new(curve, unit.speed)),
        };
    }
}

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct Health(pub f32);

impl Default for Health {
    fn default() -> Self {
        Self(100.0)
    }
}

#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Component, Default)]
pub struct Spawner {
    prefab: UnitPrefab,
    cooldown: Duration,
    #[reflect(ignore)]
    next: Duration,
}

impl Default for Spawner {
    fn default() -> Self {
        Self {
            prefab: Default::default(),
            cooldown: Duration::from_secs(1),
            next: Duration::from_secs(0),
        }
    }
}

pub fn tick_spawners(mut commands: Commands, mut spawners: Query<&mut Spawner>, time: Res<Time>) {
    let time = time.elapsed();
    for mut spawner in spawners.iter_mut() {
        if spawner.next < time {
            spawner.next = time + spawner.cooldown;
            commands
                .spawn(PrefabBundle::new(spawner.prefab.path()))
                .insert(Local);
        }
    }
}

pub fn die(mut commands: Commands, q: Query<(Entity, &Health), Changed<Health>>) {
    for (entity, health) in q.iter() {
        if health.0 <= 0.0 {
            commands.get_entity(entity).unwrap().despawn_recursive();
        }
    }
}

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<Health>()
            .editor_registry::<Unit>()
            .editor_registry::<Spawner>()
            .register_type::<UnitPrefab>()
            .add_systems(PreUpdate, instantiate_unit.in_set(Gameplay))
            .add_systems(Update, (die, tick_spawners).in_set(Gameplay));
        #[cfg(feature = "editor")]
        app.editor_bundle(
            "Prefab",
            "Unit",
            (
                SpatialBundle::default(),
                Unit::default(),
                Health::default(),
                Name::new("Unit"),
            ),
        );
        #[cfg(feature = "editor")]
        app.editor_bundle(
            "Level",
            "Spawner",
            (Spawner::default(), Name::new("Spawner")),
        );
    }
}
