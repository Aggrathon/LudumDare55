use bevy::math::cubic_splines::CubicCurve;
use bevy::prelude::*;
use space_editor::prelude::*;

use crate::state::Gameplay;

#[derive(Component, Clone)]
pub struct Curve {
    curve: CubicCurve<Vec3>,
    step: f32,
}

impl Curve {
    #[allow(unused)]
    pub fn position(&self, pos: f32, length: f32) -> (f32, Vec3) {
        let start = self.curve.position(pos * self.step);
        let l2 = length * length;
        let mut lowp = pos;
        let mut highp = pos + length;
        while start.distance_squared(self.curve.position(highp * self.step)) < l2 {
            lowp = highp;
            highp += highp - pos;
        }
        let mut midp = highp * 0.5 + lowp * 0.5;
        for _ in 0..3 {
            if start.distance_squared(self.curve.position(midp * self.step)) < l2 {
                lowp = midp;
            } else {
                highp = midp;
            }
            midp = highp * 0.5 + lowp * 0.5
        }
        (midp, self.curve.position(midp * self.step))
    }
}

impl Default for Curve {
    fn default() -> Self {
        Self {
            curve: CubicBSpline::new([]).to_curve(),
            step: 0.0,
        }
    }
}

#[derive(Component, Clone, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Spline;

/// Splines into Curves
fn convert_splines(
    mut commands: Commands,
    splines: Query<(Entity, &Children), With<Spline>>,
    nodes: Query<&GlobalTransform, With<Parent>>,
) {
    for (entity, children) in &splines {
        if let Some(curve) = get_curve(children, &nodes) {
            let divs = curve.segments().len();
            let start = curve.position(0.0);
            let length = curve
                .iter_positions(divs * 20)
                .fold((0.0, start), |(len, prev), next| {
                    (len + prev.distance(next), next)
                })
                .0;
            commands
                .entity(entity)
                .despawn_descendants()
                .remove::<Spline>()
                .insert(Curve {
                    curve,
                    step: divs as f32 / length,
                });
        }
    }
}

#[allow(unused)]
fn debug_gizmos(
    splines: Query<(&Spline, &Children)>,
    nodes: Query<&GlobalTransform, With<Parent>>,
    mut gizmos: Gizmos,
) {
    for (spline, children) in &splines {
        if let Some(curve) = get_curve(children, &nodes) {
            let divisions = curve.segments().len() * 10;
            gizmos.linestrip(curve.iter_positions(divisions), Color::WHITE);
        }
    }
}

fn get_curve(
    children: &Children,
    nodes: &Query<&GlobalTransform, With<Parent>>,
) -> Option<CubicCurve<Vec3>> {
    if children.is_empty() {
        return None;
    }
    let mut points = Vec::with_capacity(children.len() + 2);
    points.push(Vec3::ZERO);
    for &child in children.iter() {
        if let Ok(gt) = nodes.get(child) {
            // TODO transform curve into local space
            points.push(gt.translation());
        }
    }
    if points.len() > 2 {
        points[0] = points[1];
        points.push(*points.last().unwrap());
    } else {
        return None;
    }
    Some(CubicCardinalSpline::new_catmull_rom(points).to_curve())
}

#[derive(Component, Clone)]
pub struct FollowCurve {
    curve: Entity,
    speed: f32,
    along: f32,
}

impl FollowCurve {
    pub fn new(curve: Entity, speed: f32) -> Self {
        FollowCurve {
            curve,
            speed,
            along: 0.0,
        }
    }

    pub fn distance(&self) -> f32 {
        self.along
    }
}

pub fn follow_curve(
    mut query: Query<(&mut FollowCurve, &mut Transform)>,
    curves: Query<(Entity, &Curve)>,
    time: Res<Time>,
) {
    for (mut follow, mut trans) in query.iter_mut() {
        if let Ok((_, curve)) = curves.get(follow.curve) {
            let (pos, vec) = curve.position(follow.along, follow.speed * time.delta_seconds());
            follow.along = pos;
            trans.translation = vec;
        }
    }
}

pub struct SplinePlugin;

impl Plugin for SplinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreUpdate, (convert_splines).in_set(Gameplay))
            .add_systems(Update, (follow_curve).in_set(Gameplay))
            .editor_registry::<Spline>();
        #[cfg(feature = "editor")]
        app.add_systems(Update, debug_gizmos.run_if(in_state(EditorState::Editor)))
            .editor_bundle(
                "Level",
                "Spline",
                (TransformBundle::default(), Spline, Name::new("Curve")),
            );
    }
}
