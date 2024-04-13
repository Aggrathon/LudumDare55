use bevy::math::Vec3;

pub fn get_random_from_iter<T, I: Iterator<Item = T>, F: Fn() -> I>(get_iter: F) -> Option<T> {
    match get_iter().count() {
        0 => None,
        1 => get_iter().next(),
        count => get_iter().nth(fastrand::usize(0..count)),
    }
}

pub fn smooth_damp(
    current: f32,
    target: f32,
    velocity: f32,
    smooth_time: f32,
    max_speed: f32,
    delta_time: f32,
) -> (f32, f32) {
    let omega = 2.0 / smooth_time;
    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let change = current - target;

    // Clamp maximum speed
    let max_change = max_speed * smooth_time;
    let change = f32::clamp(change, -max_change, max_change);
    let next = current - change;

    let temp = (velocity + omega * change) * delta_time;
    let velocity = (velocity - omega * temp) * exp;
    let output = next + (change + temp) * exp;

    // Prevent overshooting
    if (target > current) == (output > target) {
        (target, (output - target) / delta_time)
    } else {
        (output, velocity)
    }
}

pub fn smooth_damp_vec3(
    current: Vec3,
    target: Vec3,
    velocity: f32,
    smooth_time: f32,
    max_speed: f32,
    delta_time: f32,
) -> (Vec3, f32) {
    let dist = current.distance(target);
    if dist < 1e-8 {
        return (target, 0.0);
    }
    let (pos, velocity) = smooth_damp(0.0, dist, velocity, smooth_time, max_speed, delta_time);
    (current.lerp(target, pos / dist), velocity)
}
