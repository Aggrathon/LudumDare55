use std::f32::consts::PI;

use bevy::prelude::*;
use space_editor::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct LevelLocal;

fn despawn_local(mut commands: Commands, q: Query<Entity, With<LevelLocal>>) {
    for e in q.iter() {
        commands.get_entity(e).unwrap().despawn_recursive();
    }
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
            .add_systems(PreUpdate, randomize.in_set(Gameplay));
    }
}
