use bevy::prelude::*;
use space_editor::prelude::*;

use crate::spline::{Curve, FollowCurve};
use crate::state::Local;
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
pub struct Health(f32);

impl Default for Health {
    fn default() -> Self {
        Self(100.0)
    }
}

#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Component, Default)]
pub struct Spawner {
    prefab: UnitPrefab,
    cooldown: f32,
    #[reflect(ignore)]
    next: f32,
}

impl Default for Spawner {
    fn default() -> Self {
        Self {
            prefab: Default::default(),
            cooldown: 1.0,
            next: 0.0,
        }
    }
}

pub fn tick_spawners(mut commands: Commands, mut spawners: Query<&mut Spawner>, time: Res<Time>) {
    let time = time.elapsed_seconds();
    for mut spawner in spawners.iter_mut() {
        if spawner.next < time {
            spawner.next = time + spawner.cooldown;
            commands
                .spawn(PrefabBundle::new(spawner.prefab.path()))
                .insert(Local);
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
            .add_systems(
                PreUpdate,
                instantiate_unit.run_if(in_state(EditorState::Game)),
            )
            .add_systems(Update, tick_spawners.run_if(in_state(EditorState::Game)));

        #[cfg(feature = "editor")]
        app.editor_bundle(
            "Units",
            "PlaceHolder",
            (
                SpatialBundle::default(),
                Unit::default(),
                Health::default(),
                Name::new("Placeholder"),
            ),
        );
        #[cfg(feature = "editor")]
        app.editor_bundle(
            "Units",
            "Spawner",
            (Spawner::default(), Name::new("Spawner")),
        );
    }
}
