use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use enum_iterator::{all, Sequence};
use space_editor::prelude::*;

use crate::unit::Spawner;

#[derive(Component, Clone, Copy)]
pub struct LevelLocal;

fn despawn_local(mut commands: Commands, q: Query<Entity, With<LevelLocal>>) {
    for e in q.iter() {
        commands.get_entity(e).unwrap().despawn_recursive();
    }
}

#[derive(Clone, Copy, Default, States, Debug, Hash, PartialEq, Eq, Sequence)]
pub enum Level {
    #[default]
    Unknown,
    MainMenu,
    Level0,
    Reload,
    Next,
}

impl Level {
    pub const fn next(&self) -> Self {
        match self {
            Level::Unknown => Level::MainMenu,
            Level::MainMenu => Level::Level0,
            Level::Level0 => Level::MainMenu,
            Level::Reload => Level::MainMenu,
            Level::Next => Level::MainMenu,
        }
    }

    #[allow(dead_code)]
    pub fn load_next(&mut self) {
        *self = self.next();
    }
}

fn load_level(
    mut commands: Commands,
    level: Res<State<Level>>,
    mut stats: ResMut<GameStats>,
    time: Res<Time>,
) {
    match level.get() {
        Level::Reload | Level::Next => return,
        Level::Unknown => {}
        Level::MainMenu => {
            commands
                .spawn(PrefabBundle::new("scenes/Level0.scn.ron"))
                .insert(LevelLocal);
        }
        Level::Level0 => {
            commands
                .spawn(PrefabBundle::new("scenes/Level0.scn.ron"))
                .insert(LevelLocal);
        }
    };
    *stats = GameStats::default();
    stats.start_time = time.elapsed();
    commands.spawn((LevelLocal, Spawner::default()));
}

fn reload_level(
    mut reader: EventReader<StateTransitionEvent<Level>>,
    mut next: ResMut<NextState<Level>>,
) {
    for ev in reader.read() {
        match ev.after {
            Level::Reload => next.set(ev.before),
            Level::Next => next.set(ev.before.next()),
            _ => {}
        }
    }
}

#[derive(Clone, Resource)]
pub struct GameStats {
    pub upgrade_speed: u8,
    pub upgrade_level: u8,
    pub upgrade_circle: u8,
    pub upgrade_appease: u8,
    pub souls_total: u32,
    pub souls_current: u32,
    pub souls_next: u32,
    pub defender_morale: u8,
    pub start_time: Duration,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            upgrade_speed: Default::default(),
            upgrade_level: Default::default(),
            upgrade_circle: Default::default(),
            upgrade_appease: Default::default(),
            souls_total: Default::default(),
            souls_current: Default::default(),
            souls_next: 10,
            defender_morale: Self::MAX_MORALE,
            start_time: Default::default(),
        }
    }
}

impl GameStats {
    pub const MAX_MORALE: u8 = 5;
    pub const MAX_CIRCLES: u8 = 5;
    pub const TIME_LIMIT: Duration = Duration::from_secs(180);
    pub const APPEASEMENT: Duration = Duration::from_secs(30);
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Gameplay;

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
pub struct Randomize {
    scale: f32,
    color: f32,
    angle: f32,
}

fn randomize(mut commands: Commands, mut q: Query<(Entity, &Randomize, &mut Transform)>) {
    for (entity, rnd, mut tr) in q.iter_mut() {
        tr.scale *= fastrand::f32() * rnd.scale * 2.0 - rnd.scale + 1.0;
        tr.rotate_y(fastrand::f32() * PI * 2.0);
        tr.rotate_x(fastrand::f32() * rnd.angle);
        commands.get_entity(entity).unwrap().remove::<Randomize>();
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<Randomize>()
            .add_systems(OnExit(EditorState::Game), despawn_local)
            .configure_sets(Update, Gameplay.run_if(in_state(EditorState::Game)))
            .configure_sets(PreUpdate, Gameplay.run_if(in_state(EditorState::Game)))
            .add_systems(PreUpdate, randomize.in_set(Gameplay))
            .init_state::<Level>()
            .init_resource::<GameStats>();
        for l in all::<Level>() {
            app.add_systems(OnEnter(l), (load_level, reload_level))
                .add_systems(OnExit(l), despawn_local);
        }
    }
}
