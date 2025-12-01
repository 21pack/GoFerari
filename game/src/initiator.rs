use crate::{
    world::{Camera, Unit},
    MOVEMENT_SPEEDUP,
};

use ferari::world::{Player, State, UnitMovement};

/// Returns a list of game objects that are currently visible within the camera's view.
///
/// This function filters all game units (player and mobs) to only include those
/// that fall within the camera's current field of view. The visibility is determined
/// by the camera's position and viewport dimensions.
///
/// # Arguments
///
/// * `cur_state` - The current game state containing all units
/// * `camera` - The camera that defines the visible area of the game world
///
/// # Returns
///
/// A vector containing all [`Unit`] objects that are currently visible to the camera.
/// The player unit is always included first, followed by any visible mobs.
pub fn get_visible_objects(cur_state: &State, camera: &Camera) -> Vec<Unit> {
    let mut units = Vec::new();
    units.push(cur_state.player.unit.clone());
    units.extend(cur_state.mobs.clone());

    units.into_iter().filter(|mob| camera.is_visible(mob.pixel_x, mob.pixel_y)).collect()
}

/// Performs linear interpolation between two scalar values.
///
/// Given start value `a`, end value `b`, and interpolation factor `t` (typically in `[0.0, 1.0]`),
/// this function computes a value that lies `t` percent of the way from `a` to `b`.
///
/// # Formula
///
/// ```text
/// result = a * (1 - t) + b * t
/// ```
///
/// # Arguments
///
/// * `a` - the starting value
/// * `b` - the ending value
/// * `t` - the interpolation parameter
///
/// # Returns
///
/// The interpolated value between `a` and `b`.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

/// Generates the appropriate sprite name for the player based on their current state and time.
///
/// The sprite name encodes:
/// - The animation type (e.g., `"idle"`, `"running"`, `"pushing"`)
/// - The player's facing direction (e.g., `"SE"`, `"NW"`)
/// - The current frame index within the animation cycle
///
/// The animation frame is determined by elapsed `total_time`, scaled by `MOVEMENT_SPEEDUP`
/// to maintain consistent animation speed under time-warping effects.
///
/// # Arguments
///
/// * `player` – reference to the player object containing movement and direction state
/// * `total_time` – total elapsed game time in seconds
///
/// # Returns
///
/// A string in the format `"{animation}_{direction}_{frame_index}"`, suitable for
/// loading the correct sprite from asset resources (e.g., `"running_se_5"`).
pub fn get_player_sprite(player: &Player, total_time: f64) -> String {
    let k = 1.0 / 1000.0 / MOVEMENT_SPEEDUP as f64;
    let (prefix, total_frames, period) = match player.unit.movement {
        UnitMovement::Moving { .. } => ("running", 14, 45.0 * k),
        UnitMovement::Pushing { .. } => ("pushing", 37, 25.0 * k),
        UnitMovement::Idle => ("idle", 31, 45.0 * k),
        UnitMovement::PrePushing { .. } => ("walkingforward", 24, 30.0 * k),
        UnitMovement::PostPushing { .. } => ("walkingback", 23, 30.0 * k),
    };

    let dir_suffix = player.unit.direction.as_str();

    let cycles = (total_time / period).floor() as u32;
    let frame_idx = cycles % total_frames;

    format!("{}_{}_{}", prefix, dir_suffix, frame_idx)
}
