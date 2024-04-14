use std::time::Duration;

use bevy::prelude::*;
use enum_iterator::Sequence;
use space_editor::prelude::*;

use crate::level::{GameStats, Gameplay, LevelLocal};
use crate::spline::{Curve, FollowCurve, Width};
use crate::utils::get_random_from_iter;

#[derive(Clone, Copy, Reflect, Default, PartialEq, Eq, Sequence)]
#[reflect(Default)]
pub enum UnitPrefab {
    #[default]
    Imp,
    Ghoul,
}

impl UnitPrefab {
    #[inline]
    pub const fn path(&self) -> &'static str {
        match self {
            UnitPrefab::Imp => "scenes/Imp.scn.ron",
            UnitPrefab::Ghoul => "scenes/Ghoul.scn.ron",
        }
    }

    #[inline]
    pub const fn cost(&self) -> f32 {
        match self {
            UnitPrefab::Imp => 1.0,
            UnitPrefab::Ghoul => 2.0,
        }
    }

    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            UnitPrefab::Imp => "Imp",
            UnitPrefab::Ghoul => "Ghoul",
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
    curves: Query<(Entity, Option<&Width>), With<Curve>>,
) {
    for (entity, unit) in units.iter() {
        match get_random_from_iter(|| curves.iter()) {
            None => continue,
            Some((curve, width)) => {
                let radius = width.map(|w| w.0).unwrap_or(1.0);
                commands
                    .get_entity(entity)
                    .unwrap()
                    .insert(FollowCurve::new(
                        curve,
                        unit.speed,
                        Vec3::new(
                            radius * fastrand::f32() * 2.0 - radius,
                            0.0,
                            radius * fastrand::f32() * 2.0 - radius,
                        ),
                    ));
            }
        }
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
    pub prefab: UnitPrefab,
    pub number: u8,
    #[reflect(ignore)]
    prev: Duration,
}

impl Spawner {
    #[inline]
    pub fn progress(&self, time: Duration, speed: u8) -> f32 {
        ((time - self.prev).as_secs_f32() / self.total_cooldown(speed).as_secs_f32()).min(1.0)
    }

    #[inline]
    pub fn total_cooldown(&self, speed: u8) -> Duration {
        Duration::from_secs_f32(
            self.prefab.cost() * (self.number as f32) * 5.0 / (5.0 + speed as f32),
        )
    }

    #[inline]
    pub fn next(&self, speed: u8) -> Duration {
        self.prev + self.total_cooldown(speed)
    }
}

impl Default for Spawner {
    fn default() -> Self {
        Self {
            prefab: Default::default(),
            prev: Duration::from_secs(0),
            number: 1,
        }
    }
}

pub fn tick_spawners(
    mut commands: Commands,
    mut spawners: Query<&mut Spawner>,
    time: Res<Time>,
    stats: Res<GameStats>,
) {
    let time = time.elapsed();
    for mut spawner in spawners.iter_mut() {
        if spawner.next(stats.upgrade_speed) < time {
            spawner.prev = time;
            for _ in 0..spawner.number {
                commands
                    .spawn(PrefabBundle::new(spawner.prefab.path()))
                    .insert(LevelLocal);
            }
        }
    }
}

pub fn die(
    mut commands: Commands,
    q: Query<(Entity, &Health), Changed<Health>>,
    mut stats: ResMut<GameStats>,
) {
    for (entity, health) in q.iter() {
        if health.0 <= 0.0 {
            stats.souls_current += 1;
            stats.souls_total += 1;
            commands.get_entity(entity).unwrap().despawn_recursive();
        }
    }
}

#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Component, Default)]
pub struct Goal {
    radius: f32,
}

impl Default for Goal {
    fn default() -> Self {
        Self { radius: 3.0 }
    }
}

fn score(
    mut commands: Commands,
    goals: Query<(&Goal, &GlobalTransform)>,
    units: Query<(Entity, &GlobalTransform), With<Unit>>,
    mut stats: ResMut<GameStats>,
) {
    for (goal, gt) in goals.iter() {
        let pos = gt.translation();
        let rad = goal.radius * goal.radius;
        for (e, gt) in units.iter() {
            if pos.distance_squared(gt.translation()) < rad {
                commands.get_entity(e).unwrap().despawn_recursive();
                if stats.defender_morale > 0 {
                    stats.defender_morale -= 1;
                }
            }
        }
    }
}

#[allow(dead_code)]
fn draw_gizmos(mut gizmos: Gizmos, q: Query<(&Goal, &GlobalTransform)>) {
    for (g, gt) in q.iter() {
        gizmos.circle(gt.translation(), Direction3d::Y, g.radius, Color::GREEN);
    }
}

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<Health>()
            .editor_registry::<Unit>()
            .editor_registry::<Spawner>()
            .editor_registry::<Goal>()
            .register_type::<UnitPrefab>()
            .add_systems(PreUpdate, instantiate_unit.in_set(Gameplay))
            .add_systems(Update, (die, tick_spawners, score).in_set(Gameplay));
        #[cfg(feature = "editor")]
        {
            app.add_systems(Update, draw_gizmos);
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
            app.editor_bundle(
                "Level",
                "Spawner",
                (Spawner::default(), Name::new("Spawner")),
            );
            app.editor_bundle("Level", "Goal", (Goal::default(), Name::new("Goal")));
        }
    }
}
