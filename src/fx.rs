use std::time::Duration;

use bevy::prelude::*;
use space_editor::prelude::*;

use crate::level::{Gameplay, LevelLocal};

#[derive(Clone, Copy, Reflect, Default)]
#[reflect(Default)]
pub enum FxLibrary {
    #[default]
    Death,
    Explosion,
}

impl FxLibrary {
    pub fn path(&self) -> &'static str {
        match self {
            FxLibrary::Death => "scenes/DeathFx.scn.ron",
            FxLibrary::Explosion => "scenes/Explosion.scn.ron",
        }
    }
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
pub struct Spawnable(pub FxLibrary);

impl Spawnable {
    pub fn spawn(&self, pos: Vec3, commands: &mut Commands) {
        commands
            .spawn(PrefabBundle::new(self.0.path()))
            .insert((LevelLocal, Transform::from_translation(pos)));
    }
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component, Default)]
pub struct DespawnTimer(pub Duration);

fn despawn_timer(mut commands: Commands, q: Query<(Entity, &DespawnTimer)>, time: Res<Time>) {
    let time = time.elapsed();
    for (e, t) in q.iter() {
        if t.0 < time {
            commands.get_entity(e).unwrap().despawn_recursive();
        }
    }
}

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FxLibrary>()
            .editor_registry::<DespawnTimer>()
            .editor_registry::<Spawnable>()
            .add_systems(Update, despawn_timer.in_set(Gameplay));
    }
}
