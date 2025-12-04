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

#[cfg(test)]
mod visible_objects_tests {
    use super::*;

    #[derive(Clone)]
    struct DummyUnit {
        x: f32,
        y: f32,
    }
    impl DummyUnit {
        fn to_real_unit(&self) -> Unit {
            Unit {
                pixel_x: self.x,
                pixel_y: self.y,
                tile_x: 0,
                tile_y: 0,
                x_speed: 0.0,
                y_speed: 0.0,
                movement: UnitMovement::default(),
                direction: ferari::world::Direction::default(),
            }
        }
    }

    #[derive(Clone)]
    struct DummyState {
        player: DummyUnit,
        mobs: Vec<DummyUnit>,
    }

    impl DummyState {
        fn to_real_state(&self) -> State {
            State {
                player: Player { unit: self.player.to_real_unit() },
                mobs: self.mobs.iter().map(|m| m.to_real_unit()).collect(),
                mob_grid: vec![None; 128 * 128],
                grid_width: 128,
            }
        }
    }

    #[test]
    fn test_get_visible_objects_player_included() {
        let dummy_state = DummyState { player: DummyUnit { x: 0.0, y: 0.0 }, mobs: vec![] };
        let state = dummy_state.to_real_state();

        let camera = Camera::new(0.0, 0.0, 800, 600);
        let visible = get_visible_objects(&state, &camera);

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].pixel_x, state.player.unit.pixel_x);
        assert_eq!(visible[0].pixel_y, state.player.unit.pixel_y);
    }

    #[test]
    fn test_get_visible_objects_mobs_visible() {
        let dummy_state = DummyState {
            player: DummyUnit { x: 0.0, y: 0.0 },
            mobs: vec![DummyUnit { x: 10.0, y: 10.0 }, DummyUnit { x: 1000.0, y: 1000.0 }],
        };
        let state = dummy_state.to_real_state();
        let camera = Camera::new(0.0, 0.0, 50, 50);

        let visible = get_visible_objects(&state, &camera);
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[1].pixel_x, 10.0);
        assert_eq!(visible[1].pixel_y, 10.0);
    }

    #[test]
    fn test_get_visible_objects_mobs_outside_not_included() {
        let dummy_state = DummyState {
            player: DummyUnit { x: 0.0, y: 0.0 },
            mobs: vec![DummyUnit { x: 100.0, y: 100.0 }],
        };
        let state = dummy_state.to_real_state();
        let camera = Camera::new(0.0, 0.0, 50, 50);

        let visible = get_visible_objects(&state, &camera);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].pixel_x, state.player.unit.pixel_x);
    }

    #[test]
    fn test_get_visible_objects_multiple_mobs() {
        let dummy_state = DummyState {
            player: DummyUnit { x: 0.0, y: 0.0 },
            mobs: vec![
                DummyUnit { x: 5.0, y: 5.0 },
                DummyUnit { x: 20.0, y: 20.0 },
                DummyUnit { x: 100.0, y: 100.0 },
            ],
        };
        let state = dummy_state.to_real_state();
        let camera = Camera::new(0.0, 0.0, 50, 50);

        let visible = get_visible_objects(&state, &camera);
        assert_eq!(visible.len(), 3);
        let positions: Vec<_> = visible.iter().map(|u| (u.pixel_x, u.pixel_y)).collect();
        assert!(positions.contains(&(0.0, 0.0)));
        assert!(positions.contains(&(5.0, 5.0)));
        assert!(positions.contains(&(20.0, 20.0)));
    }
}

#[cfg(test)]
mod lerp_tests {
    use super::*;

    #[test]
    fn test_lerp_basic() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn test_lerp_negative_values() {
        assert_eq!(lerp(-10.0, 10.0, 0.5), 0.0);
        assert_eq!(lerp(-10.0, 0.0, 0.5), -5.0);
        assert_eq!(lerp(10.0, -10.0, 0.5), 0.0);
    }

    #[test]
    fn test_lerp_extreme_t() {
        assert_eq!(lerp(0.0, 10.0, 2.0), 20.0); // t > 1.0
        assert_eq!(lerp(0.0, 10.0, -1.0), -10.0); // t < 0.0
    }
}

#[cfg(test)]
mod player_sprite_tests {
    use super::*;
    use ferari::world::Direction;

    #[derive(Clone)]
    struct DummyPlayer {
        movement: UnitMovement,
        direction: Direction,
    }
    impl DummyPlayer {
        fn to_real_player(&self) -> Player {
            Player {
                unit: Unit {
                    pixel_x: 0.0,
                    pixel_y: 0.0,
                    tile_x: 0,
                    tile_y: 0,
                    x_speed: 0.0,
                    y_speed: 0.0,
                    movement: self.movement.clone(),
                    direction: self.direction.clone(),
                },
            }
        }
    }

    fn create_test_movement(movement_type: &str) -> UnitMovement {
        match movement_type {
            "moving" => UnitMovement::Moving {
                start_x: 0.0,
                start_y: 0.0,
                target_x: 1.0,
                target_y: 1.0,
                elapsed_time: 0.0,
                duration: 1.0,
            },
            "pushing" => UnitMovement::Pushing {
                start_x: 0.0,
                start_y: 0.0,
                target_x: 1.0,
                target_y: 1.0,
                elapsed_time: 0.0,
                duration: 1.0,
                recoil_target_x: 0.0,
                recoil_target_y: 0.0,
            },
            "pre_pushing" => UnitMovement::PrePushing {
                start_x: 0.0,
                start_y: 0.0,
                target_x: 1.0,
                target_y: 1.0,
                elapsed_time: 0.0,
                duration: 1.0,
                box_idx: 0,
                player_next_tx: 0,
                player_next_ty: 0,
                box_next_tx: 0,
                box_next_ty: 0,
                push_dx: 0,
                push_dy: 0,
            },
            "post_pushing" => UnitMovement::PostPushing {
                start_x: 0.0,
                start_y: 0.0,
                target_x: 1.0,
                target_y: 1.0,
                elapsed_time: 0.0,
                duration: 1.0,
            },
            "idle" => UnitMovement::Idle,
            _ => panic!("Unknown movement type"),
        }
    }

    #[test]
    fn test_get_player_sprite_running() {
        let period = 45.0 * (1.0 / 1000.0);
        let dummy_player =
            DummyPlayer { movement: create_test_movement("moving"), direction: Direction::SE };
        let player = dummy_player.to_real_player();

        let sprite = get_player_sprite(&player, 0.0);
        assert_eq!(sprite, "running_se_0");

        let sprite1 = get_player_sprite(&player, period * 13.0);
        assert_eq!(sprite1, "running_se_13");

        let sprite2 = get_player_sprite(&player, period * 14.0);
        assert_eq!(sprite2, "running_se_0");

        let sprite3 = get_player_sprite(&player, period * 27.0);
        assert_eq!(sprite3, "running_se_12");
    }

    #[test]
    fn test_get_player_sprite_pushing() {
        let period = 25.0 * (1.0 / 1000.0);
        let dummy_player =
            DummyPlayer { movement: create_test_movement("pushing"), direction: Direction::SE };
        let player = dummy_player.to_real_player();

        let sprite = get_player_sprite(&player, 0.0);
        assert_eq!(sprite, "pushing_se_0");

        let sprite1 = get_player_sprite(&player, period * 1.0);
        assert_eq!(sprite1, "pushing_se_1");

        let sprite2 = get_player_sprite(&player, period * 3.0);
        assert_eq!(sprite2, "pushing_se_3");

        let sprite3 = get_player_sprite(&player, period * 37.0);
        assert_eq!(sprite3, "pushing_se_0");
    }
}
