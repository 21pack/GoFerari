use crate::{initiator::lerp, input::InputSnapshot, TILE_SIZE};

use ferari::world::{PlayerMovement, State};

// TODO: rm?
// /// Calculates the absolute value (length) of a 2D vector.
// ///
// /// # Arguments
// /// * `vec` - A tuple representing a 2D vector (x, y)
// ///
// /// # Returns
// /// * The length of the vector as f32
// fn abs_vector(vec: (f32, f32)) -> f32 {
//     let (dx, dy) = vec;
//     (dx * dx + dy * dy).sqrt()
// }

// /// Normalizes a 2D vector to unit length.
// ///
// /// If the vector's length is not more than 0.1, returns a zero vector
// /// to avoid division by very small numbers.
// ///
// /// # Arguments
// /// * `vec` - A tuple representing a 2D vector (x, y)
// ///
// /// # Returns
// /// * A normalized vector as tuple (x, y) or zero vector if length is small
// fn normalize_vector(vec: (f32, f32)) -> (f32, f32) {
//     let (dx, dy) = vec;
//     let distance = abs_vector(vec);
//     if distance > 0.1 {
//         (dx / distance, dy / distance)
//     } else {
//         (0., 0.)
//     }
// }

/// Updates the game state for one simulation step.
///
/// Handles player movement based on input and mob behaviour.
///
/// # Arguments
/// * `curr_state` - Mutable reference to the current game state
/// * `input_state` - Reference to the current input snapshot
pub fn make_step(
    curr_state: &mut State,
    input_state: &InputSnapshot,
    delta: f32,
    game: &ferari::assets::GameMap,
) {
    const MOVE_DURATION: f32 = 0.3;

    let player = &mut curr_state.player;

    match &mut player.movement {
        PlayerMovement::Idle => {
            let (mut dx, mut dy) = (0, 0);

            if input_state.right {
                dx = 1;
                dy = 0;
            } else if input_state.left {
                dx = -1;
                dy = 0;
            } else if input_state.up {
                dx = 0;
                dy = -1;
            } else if input_state.down {
                dx = 0;
                dy = 1;
            }

            if dx == 0 && dy == 0 {
                return;
            }

            let next_tx = player.unit.tile_x + dx;
            let next_ty = player.unit.tile_y + dy;

            if !game.is_walkable(next_tx, next_ty) {
                return;
            }

            let step_x = (TILE_SIZE as f32) * 0.5;
            let step_y = (TILE_SIZE as f32) * 0.25;

            let mut target_px = player.unit.pixel_x;
            let mut target_py = player.unit.pixel_y;

            match (dx, dy) {
                (1, 0) => {
                    // right
                    target_px += step_x;
                    target_py += step_y;
                }
                (-1, 0) => {
                    // left
                    target_px -= step_x;
                    target_py -= step_y;
                }
                (0, -1) => {
                    // up
                    target_px += step_x;
                    target_py -= step_y;
                }
                (0, 1) => {
                    // down
                    target_px -= step_x;
                    target_py += step_y;
                }
                _ => {}
            }

            player.movement = PlayerMovement::Moving {
                start_x: player.unit.pixel_x,
                start_y: player.unit.pixel_y,
                target_x: target_px,
                target_y: target_py,
                elapsed_time: 0.0,
                duration: MOVE_DURATION,
            };

            player.unit.tile_x = next_tx;
            player.unit.tile_y = next_ty;
        }
        PlayerMovement::Moving { start_x, start_y, target_x, target_y, elapsed_time, duration } => {
            *elapsed_time += delta;
            let progress = (*elapsed_time / *duration).min(1.0);

            player.unit.pixel_x = lerp(*start_x, *target_x, progress);
            player.unit.pixel_y = lerp(*start_y, *target_y, progress);

            if progress >= 1.0 {
                player.unit.pixel_x = *target_x;
                player.unit.pixel_y = *target_y;

                player.movement = PlayerMovement::Idle;
            }
        } // PlayerMovement::Pushing {
          //     start_x,
          //     start_y,
          //     target_x,
          //     target_y,
          //     elapsed_time,
          //     duration,
          // } => todo!(),
    }
}

// TODO: rewrite
// #[cfg(test)]
// mod tests {
//     use super::State;
//     use crate::assets::GameMap;

//     use super::*;

//     #[test]
//     fn test_abs_vector_zero() {
//         assert_eq!(abs_vector((0.0, 0.0)), 0.0);
//     }

//     #[test]
//     fn test_abs_vector_nonzero() {
//         let len = abs_vector((3.0, 4.0));
//         assert!((len - 5.0).abs() < 1e-5);
//     }

//     #[test]
//     fn test_normalize_vector_basic() {
//         let n = normalize_vector((3.0, 4.0));
//         assert!(((n.0 * n.0 + n.1 * n.1).sqrt() - 1.0).abs() < 1e-5);
//     }

//     #[test]
//     fn test_normalize_vector_small_vector_returns_zero() {
//         let n = normalize_vector((0.01, 0.01));
//         assert_eq!(n, (0.0, 0.0));
//     }

//     fn make_test_state() -> State {
//         let game_map = GameMap::load("input.json").expect("failed to load game map for tests");

//         let mut state = State::new(&game_map);

//         state.player.x = 0.0;
//         state.player.y = 0.0;
//         state.player.x_speed = 0.0;
//         state.player.y_speed = 0.0;

//         if state.mobs.is_empty() {
//             state.mobs.push(crate::world::Unit {
//                 x: 100.0,
//                 y: 0.0,
//                 x_speed: -0.5,
//                 y_speed: 0.0,
//                 ..Default::default()
//             });
//         }

//         state
//     }

//     #[test]
//     fn test_player_moves_right() {
//         let mut state = make_test_state();

//         let input = crate::input::InputSnapshot {
//             up: false,
//             down: false,
//             left: false,
//             right: true,
//             escape: false,
//         };

//         make_step(&mut state, &input);

//         assert!((state.player.x - 0.75).abs() < 1e-5);
//         assert!((state.player.y - 0.0).abs() < 1e-5);
//     }

//     #[test]
//     fn test_player_moves_up_left_diagonal() {
//         let mut state = make_test_state();

//         let input = crate::input::InputSnapshot {
//             up: true,
//             down: false,
//             left: true,
//             right: false,
//             escape: false,
//         };

//         make_step(&mut state, &input);

//         let dx = state.player.x;
//         let dy = state.player.y;
//         let len = (dx * dx + dy * dy).sqrt();
//         assert!((len - 0.75).abs() < 1e-5);
//         assert!(dx < 0.0 && dy < 0.0);
//     }

//     #[test]
//     fn test_mob_moves_toward_player() {
//         let mut state = make_test_state();
//         state.mobs[0].x = 50.0;
//         state.mobs[0].y = 0.0;
//         state.mobs[0].x_speed = -0.5;
//         state.mobs[0].y_speed = 0.0;

//         let input = crate::input::InputSnapshot {
//             up: false,
//             down: false,
//             left: false,
//             right: false,
//             escape: false,
//         };

//         make_step(&mut state, &input);

//         assert!(state.mobs[0].x < 50.0);
//         assert!(state.mobs[0].y.abs() < 1e-3);
//     }

//     #[test]
//     fn test_collision_pushes_mob_back() {
//         let mut state = make_test_state();

//         state.mobs[0].x = 2.0;
//         state.mobs[0].y = 0.0;

//         let input = crate::input::InputSnapshot {
//             up: false,
//             down: false,
//             left: false,
//             right: false,
//             escape: false,
//         };

//         make_step(&mut state, &input);

//         let vec_from = (state.mobs[0].x - state.player.x, state.mobs[0].y - state.player.y);
//         let dist = (vec_from.0 * vec_from.0 + vec_from.1 * vec_from.1).sqrt();
//         assert!((dist - 10.0).abs() < 1e-3);
//     }
// }
