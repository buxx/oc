use glam::Vec3;
// FIXME BS NOW: use fastrand ?
use rand::RngExt;

// WARNING: AI generated method
/// Computes a shooting direction from start to end, with inaccuracy applied.
///
/// `spread` is the maximum deviation angle (in degrees) from the
/// perfect aim direction. 0.0 = perfect accuracy, higher = less accurate.
/// A soldier who is bad at aiming might have spread = 5.0-15.0,
/// while a sniper might use 0.1-1.0.
pub fn direction_with_inaccuracy(original: Vec3, spread: f32) -> Vec3 {
    if spread <= 0.0 || original == Vec3::ZERO {
        return original;
    }

    let mut rng = rand::rng();

    // Build an orthonormal basis around perfect_direction so we can
    // rotate it randomly within a cone.
    let arbitrary = if original.x.abs() < 0.99 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let tangent = original.cross(arbitrary).normalize();
    let bitangent = original.cross(tangent);

    let max_angle_rad = spread.to_radians();

    // Uniform sampling over the spherical cap defined by max_angle_rad.
    // Using cos(theta) uniformly distributed gives uniform area sampling
    // on the cap (avoids bunching near the center or edge).
    let range = (max_angle_rad.cos())..1.0;
    if range.is_empty() {
        return original;
    }
    let cos_theta = rng.random_range(range);
    let theta = cos_theta.acos();
    let phi = rng.random_range(0.0..std::f32::consts::TAU);

    let sin_theta = theta.sin();
    let random_offset = tangent * (sin_theta * phi.cos())
        + bitangent * (sin_theta * phi.sin())
        + original * cos_theta;

    random_offset.normalize_or_zero()
}
