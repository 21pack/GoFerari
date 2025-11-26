use crate::{initiator::lerp, input::InputSnapshot, TILE_SIZE};

use ferari::world::{Direction, State, UnitMovement};

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
    const BOX_DRAG: f32 = 2.5;

    let map_width = game.size[0] as usize;

    // ============================================
    // ANIMATION UPDATE (player + all mobs)
    // ============================================

    // Player update
    let mut player_is_busy = false;
    {
        let unit = &mut curr_state.player.unit;
        match &mut unit.movement {
            UnitMovement::Moving {
                start_x,
                start_y,
                target_x,
                target_y,
                elapsed_time,
                duration,
            }
            | UnitMovement::Pushing {
                start_x,
                start_y,
                target_x,
                target_y,
                elapsed_time,
                duration,
            } => {
                *elapsed_time += delta;
                let progress = (*elapsed_time / *duration).min(1.0);

                unit.pixel_x = lerp(*start_x, *target_x, progress);
                unit.pixel_y = lerp(*start_y, *target_y, progress);

                if progress >= 1.0 {
                    unit.pixel_x = *target_x;
                    unit.pixel_y = *target_y;
                    unit.movement = UnitMovement::Idle;
                } else {
                    player_is_busy = true;
                }
            }
            UnitMovement::Idle => {}
        }
    }

    // Mob (box) update
    for unit in curr_state.mobs.iter_mut() {
        match &mut unit.movement {
            UnitMovement::Moving {
                start_x,
                start_y,
                target_x,
                target_y,
                elapsed_time,
                duration,
            } => {
                *elapsed_time += delta;
                let progress = (*elapsed_time / *duration).min(1.0);

                unit.pixel_x = lerp(*start_x, *target_x, progress);
                unit.pixel_y = lerp(*start_y, *target_y, progress);

                if progress >= 1.0 {
                    unit.pixel_x = *target_x;
                    unit.pixel_y = *target_y;
                    unit.movement = UnitMovement::Idle;
                }
            }
            _ => {}
        }
    }

    // If the player is busy (the animation has not finished), the input is not processed
    if player_is_busy {
        return;
    }

    // ============================================
    // INPUT PROCESSING
    // ============================================

    let (mut dx, mut dy) = (0, 0);
    let mut new_dir = curr_state.player.unit.direction;

    if input_state.right {
        dx = 1;
        dy = 0;
        new_dir = Direction::SE;
    } else if input_state.left {
        dx = -1;
        dy = 0;
        new_dir = Direction::NW;
    } else if input_state.up {
        dx = 0;
        dy = -1;
        new_dir = Direction::NE;
    } else if input_state.down {
        dx = 0;
        dy = 1;
        new_dir = Direction::SW;
    }

    curr_state.player.unit.direction = new_dir;

    if dx == 0 && dy == 0 {
        return;
    }

    // Current player coordinates
    let p_tx = curr_state.player.unit.tile_x;
    let p_ty = curr_state.player.unit.tile_y;

    // Target player coordinates
    let next_tx = p_tx + dx;
    let next_ty = p_ty + dy;

    // Helper for calculating isometric pixels
    let calc_target_pixels = |curr_x: f32, curr_y: f32, dir_x: i32, dir_y: i32| -> (f32, f32) {
        let step_x = (TILE_SIZE as f32) * 0.5;
        let step_y = (TILE_SIZE as f32) * 0.25;

        match (dir_x, dir_y) {
            (1, 0) => (curr_x + step_x, curr_y + step_y), // Right (SE)
            (-1, 0) => (curr_x - step_x, curr_y - step_y), // Left (NW)
            (0, -1) => (curr_x + step_x, curr_y - step_y), // Up (NE)
            (0, 1) => (curr_x - step_x, curr_y + step_y), // Down (SW)
            _ => (curr_x, curr_y),
        }
    };

    // ============================================
    // COLLISION AND MOVEMENT LOGIC
    // ============================================

    // Checking map boundaries and static walls
    if !game.is_walkable(next_tx, next_ty) || game.has_collidable_object_at(next_tx, next_ty) {
        return;
    }

    // Checking dynamic objects
    let next_idx = (next_ty as usize) * map_width + (next_tx as usize);
    let mob_in_front = curr_state.mob_grid.get(next_idx).copied().flatten();

    if let Some(box_idx) = mob_in_front {
        // Trying to push the box

        let behind_tx = next_tx + dx;
        let behind_ty = next_ty + dy;

        // Checking the statics behind the box
        if !game.is_walkable(behind_tx, behind_ty)
            || game.has_collidable_object_at(behind_tx, behind_ty)
        {
            return;
        }

        // Checking the dynamics behind the box
        let behind_idx = (behind_ty as usize) * map_width + (behind_tx as usize);
        if let Some(Some(_)) = curr_state.mob_grid.get(behind_idx) {
            return;
        }

        // Perform pushing

        // Update the Grid (move the box index)
        curr_state.mob_grid[next_idx] = None; // Free the cell where the box was
        curr_state.mob_grid[behind_idx] = Some(box_idx); // Occupy a new cell

        // Start the box animation
        {
            let box_unit = &mut curr_state.mobs[box_idx];
            let (bx, by) = (box_unit.pixel_x, box_unit.pixel_y);
            let (target_bx, target_by) = calc_target_pixels(bx, by, dx, dy);

            box_unit.movement = UnitMovement::Moving {
                start_x: bx,
                start_y: by,
                target_x: target_bx,
                target_y: target_by,
                elapsed_time: 0.0,
                duration: MOVE_DURATION * BOX_DRAG,
            };

            box_unit.tile_x = behind_tx;
            box_unit.tile_y = behind_ty;
        }

        // Start the player animation (Pushing)
        {
            let player = &mut curr_state.player.unit;
            let (px, py) = (player.pixel_x, player.pixel_y);
            let (target_px, target_py) = calc_target_pixels(px, py, dx, dy);

            player.movement = UnitMovement::Pushing {
                start_x: px,
                start_y: py,
                target_x: target_px,
                target_y: target_py,
                elapsed_time: 0.0,
                duration: MOVE_DURATION * BOX_DRAG,
            };
            player.tile_x = next_tx;
            player.tile_y = next_ty;
        }
    } else {
        // Normal movement (no box in front)

        let player = &mut curr_state.player.unit;
        let (px, py) = (player.pixel_x, player.pixel_y);
        let (target_px, target_py) = calc_target_pixels(px, py, dx, dy);

        player.movement = UnitMovement::Moving {
            start_x: px,
            start_y: py,
            target_x: target_px,
            target_y: target_py,
            elapsed_time: 0.0,
            duration: MOVE_DURATION,
        };
        player.tile_x = next_tx;
        player.tile_y = next_ty;
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
