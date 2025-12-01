use crate::{initiator::lerp, input::InputSnapshot, MOVEMENT_SPEEDUP, TILE_SIZE};

use ferari::world::{Direction, State, Unit, UnitMovement};

// Helper to returns (x, y) offset for a specific direction and magnitude. magnitude = 1.0 is one tile
fn get_offset(dir_x: i32, dir_y: i32, magnitude: f32) -> (f32, f32) {
    let tile_w = TILE_SIZE as f32;
    let tile_h = (TILE_SIZE as f32) * 0.5;

    let step_x = tile_w * 0.5 * magnitude;
    let step_y = tile_h * 0.5 * magnitude;

    match (dir_x, dir_y) {
        (1, 0) => (step_x, step_y),
        (-1, 0) => (-step_x, -step_y),
        (0, -1) => (step_x, -step_y),
        (0, 1) => (-step_x, step_y),
        _ => (0.0, 0.0),
    }
}

// Helper to map enum Direction to (dx, dy)
fn get_dir_delta(dir: Direction) -> (i32, i32) {
    match dir {
        Direction::SE => (1, 0),
        Direction::NW => (-1, 0),
        Direction::NE => (0, -1),
        Direction::SW => (0, 1),
    }
}

fn update_player_animation(
    unit: &mut Unit,
    delta: f32,
    input_state: &InputSnapshot,
    mob_grid: &[Option<usize>],
    game: &ferari::assets::GameMap,
    map_width: usize,
) -> (bool, Option<(usize, i32, i32, i32, i32, i32, i32)>, Option<(f32, f32, f32, f32)>) {
    let mut player_is_busy = false;

    // Walking between phases of approaching or going back to box/tile.
    let mut transition_to_push: Option<(usize, i32, i32, i32, i32, i32, i32)> = None;
    let mut transition_to_post: Option<(f32, f32, f32, f32)> = None;

    match &mut unit.movement {
        UnitMovement::Moving { start_x, start_y, target_x, target_y, elapsed_time, duration } => {
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

        UnitMovement::PrePushing {
            start_x,
            start_y,
            target_x,
            target_y,
            elapsed_time,
            duration,
            box_idx,
            player_next_tx,
            player_next_ty,
            box_next_tx,
            box_next_ty,
            push_dx,
            push_dy,
        } => {
            *elapsed_time += delta;
            let progress = (*elapsed_time / *duration).min(1.0);
            unit.pixel_x = lerp(*start_x, *target_x, progress);
            unit.pixel_y = lerp(*start_y, *target_y, progress);

            if progress >= 1.0 {
                transition_to_push = Some((
                    *box_idx,
                    *player_next_tx,
                    *player_next_ty,
                    *box_next_tx,
                    *box_next_ty,
                    *push_dx,
                    *push_dy,
                ));
                unit.pixel_x = *target_x;
                unit.pixel_y = *target_y;
                player_is_busy = true;
            } else {
                player_is_busy = true;
            }
        }

        UnitMovement::Pushing {
            start_x,
            start_y,
            target_x,
            target_y,
            elapsed_time,
            duration,
            recoil_target_x,
            recoil_target_y,
        } => {
            *elapsed_time += delta;
            let progress = (*elapsed_time / *duration).min(1.0);
            unit.pixel_x = lerp(*start_x, *target_x, progress);
            unit.pixel_y = lerp(*start_y, *target_y, progress);

            if progress >= 1.0 {
                unit.pixel_x = *target_x;
                unit.pixel_y = *target_y;

                let (dx, dy) = get_dir_delta(unit.direction);

                let continuing_input = match unit.direction {
                    Direction::SE => input_state.right,
                    Direction::NW => input_state.left,
                    Direction::NE => input_state.up,
                    Direction::SW => input_state.down,
                };

                let mut can_continue = false;

                if continuing_input {
                    let current_p_tx = unit.tile_x;
                    let current_p_ty = unit.tile_y;

                    let next_box_tx = current_p_tx + (dx * 2);
                    let next_box_ty = current_p_ty + (dy * 2);

                    let next_p_tx = current_p_tx + dx;
                    let next_p_ty = current_p_ty + dy;

                    let walkable = game.is_walkable(next_box_tx, next_box_ty)
                        && !game.has_collidable_object_at(next_box_tx, next_box_ty);

                    if next_box_ty >= 0 && next_box_tx >= 0 {
                        let next_box_idx =
                            (next_box_ty as usize) * map_width + (next_box_tx as usize);
                        let mob_blocking = mob_grid.get(next_box_idx).copied().flatten().is_some();

                        if walkable && !mob_blocking {
                            let current_box_pos_idx =
                                (next_p_ty as usize) * map_width + (next_p_tx as usize);

                            if let Some(box_idx) = mob_grid[current_box_pos_idx] {
                                can_continue = true;
                                transition_to_push = Some((
                                    box_idx,
                                    next_p_tx,
                                    next_p_ty,
                                    next_box_tx,
                                    next_box_ty,
                                    dx,
                                    dy,
                                ));
                            }
                        }
                    }
                }

                if !can_continue {
                    transition_to_post =
                        Some((*target_x, *target_y, *recoil_target_x, *recoil_target_y));
                }

                player_is_busy = true;
            } else {
                player_is_busy = true;
            }
        }

        UnitMovement::PostPushing {
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

    (player_is_busy, transition_to_push, transition_to_post)
}

fn apply_push_transition(
    state: &mut State,
    box_idx: usize,
    p_tx: i32,
    p_ty: i32,
    b_tx: i32,
    b_ty: i32,
    dx: i32,
    dy: i32,
    map_width: usize,
    delta: f32,
    box_duration: f32,
    push_duration: f32,
) {
    let old_box_idx = (p_ty as usize) * map_width + (p_tx as usize);
    let new_box_idx = (b_ty as usize) * map_width + (b_tx as usize);

    if state.mob_grid[old_box_idx] == Some(box_idx) {
        state.mob_grid[old_box_idx] = None;
        state.mob_grid[new_box_idx] = Some(box_idx);

        let box_unit = &mut state.mobs[box_idx];
        let (bx, by) = (box_unit.pixel_x.round(), box_unit.pixel_y.round());
        let (offset_x, offset_y) = get_offset(dx, dy, 1.0);
        let (target_bx, target_by) = (bx + offset_x, by + offset_y);

        box_unit.movement = UnitMovement::Moving {
            start_x: bx,
            start_y: by,
            target_x: target_bx,
            target_y: target_by,
            elapsed_time: 0.0 - delta,
            duration: box_duration,
        };
        box_unit.tile_x = b_tx;
        box_unit.tile_y = b_ty;

        let (over_x, over_y) = get_offset(dx, dy, 0.5);
        let push_target_x = bx + over_x;
        let push_target_y = by + over_y;

        let settle_x = bx;
        let settle_y = by;

        let player = &mut state.player.unit;
        player.movement = UnitMovement::Pushing {
            start_x: player.pixel_x,
            start_y: player.pixel_y,
            target_x: push_target_x,
            target_y: push_target_y,
            elapsed_time: 0.0,
            duration: push_duration,
            recoil_target_x: settle_x,
            recoil_target_y: settle_y,
        };
        player.tile_x = p_tx;
        player.tile_y = p_ty;
    }
}

fn apply_post_push_transition(
    player: &mut Unit,
    start_x: f32,
    start_y: f32,
    target_x: f32,
    target_y: f32,
    duration: f32,
) {
    player.movement = UnitMovement::PostPushing {
        start_x,
        start_y,
        target_x,
        target_y,
        elapsed_time: 0.0,
        duration: duration,
    };
}

fn update_mob_animations(mobs: &mut [Unit], delta: f32) {
    for unit in mobs.iter_mut() {
        if let UnitMovement::Moving {
            start_x,
            start_y,
            target_x,
            target_y,
            elapsed_time,
            duration,
        } = &mut unit.movement
        {
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
    }
}

fn start_pre_push_animation(
    player: &mut Unit,
    dx: i32,
    dy: i32,
    box_idx: usize,
    player_next_tx: i32,
    player_next_ty: i32,
    box_next_tx: i32,
    box_next_ty: i32,
    duration: f32,
) {
    let (px, py) = (player.pixel_x, player.pixel_y);
    let (offset_x, offset_y) = get_offset(dx, dy, 0.5);
    let mid_x = px + offset_x;
    let mid_y = py + offset_y;

    player.movement = UnitMovement::PrePushing {
        start_x: px,
        start_y: py,
        target_x: mid_x,
        target_y: mid_y,
        elapsed_time: 0.0,
        duration: duration,
        box_idx,
        player_next_tx,
        player_next_ty,
        box_next_tx,
        box_next_ty,
        push_dx: dx,
        push_dy: dy,
    };
}

fn start_walking_animation(
    player: &mut Unit,
    dx: i32,
    dy: i32,
    next_tx: i32,
    next_ty: i32,
    duration: f32,
) {
    let (px, py) = (player.pixel_x, player.pixel_y);
    let (offset_x, offset_y) = get_offset(dx, dy, 1.0);

    player.movement = UnitMovement::Moving {
        start_x: px,
        start_y: py,
        target_x: px + offset_x,
        target_y: py + offset_y,
        elapsed_time: 0.0,
        duration: duration,
    };
    player.tile_x = next_tx;
    player.tile_y = next_ty;
}

pub fn make_step(
    curr_state: &mut State,
    input_state: &InputSnapshot,
    delta: f32,
    game: &ferari::assets::GameMap,
) -> Option<u32> {
    const MOVE_DURATION: f32 = 0.35 / MOVEMENT_SPEEDUP;
    const PRE_PUSH_DURATION: f32 = 0.40 / MOVEMENT_SPEEDUP;
    const POST_PUSH_DURATION: f32 = 0.50;
    const BOX_MOVE_DURATION: f32 = 1.25 / MOVEMENT_SPEEDUP;
    const PUSH_DURATION: f32 = BOX_MOVE_DURATION;

    let map_width = game.size[0] as usize;

    // ============================================
    // ANIMATION UPDATE (player + all mobs)
    // ============================================

    // Player update
    let (mut player_is_busy, transition_to_push, transition_to_post) = update_player_animation(
        &mut curr_state.player.unit,
        delta,
        input_state,
        &curr_state.mob_grid,
        game,
        map_width,
    );

    if let Some((box_idx, p_tx, p_ty, b_tx, b_ty, dx, dy)) = transition_to_push {
        player_is_busy = true;

        apply_push_transition(
            curr_state,
            box_idx,
            p_tx,
            p_ty,
            b_tx,
            b_ty,
            dx,
            dy,
            map_width,
            delta,
            BOX_MOVE_DURATION,
            PUSH_DURATION,
        );
    }

    if let Some((sx, sy, tx, ty)) = transition_to_post {
        player_is_busy = true;

        apply_post_push_transition(&mut curr_state.player.unit, sx, sy, tx, ty, POST_PUSH_DURATION);
    }

    // Mob (box) update
    update_mob_animations(&mut curr_state.mobs, delta);

    // ============================================
    // INPUT PROCESSING
    // ============================================

    if input_state.left && input_state.right {
        return Some(0);
    }

    // If the player is busy (the animation has not finished), the input is not processed
    if player_is_busy {
        return None;
    }

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
        return None;
    }

    // Current player coordinates
    let p_tx = curr_state.player.unit.tile_x;
    let p_ty = curr_state.player.unit.tile_y;

    // Target player coordinates
    let next_tx = p_tx + dx;
    let next_ty = p_ty + dy;

    // ============================================
    // COLLISION AND MOVEMENT LOGIC
    // ============================================

    // Checking map boundaries and static walls
    if !game.is_walkable(next_tx, next_ty) || game.has_collidable_object_at(next_tx, next_ty) {
        return None;
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
            return None;
        }

        // Checking the dynamics behind the box
        let behind_idx = (behind_ty as usize) * map_width + (behind_tx as usize);
        if curr_state.mob_grid.get(behind_idx).copied().flatten().is_some() {
            return None;
        }

        // perform pre push
        start_pre_push_animation(
            &mut curr_state.player.unit,
            dx,
            dy,
            box_idx,
            next_tx,
            next_ty,
            behind_tx,
            behind_ty,
            PRE_PUSH_DURATION,
        );
    } else {
        start_walking_animation(
            &mut curr_state.player.unit,
            dx,
            dy,
            next_tx,
            next_ty,
            MOVE_DURATION,
        );
    }

    None
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
