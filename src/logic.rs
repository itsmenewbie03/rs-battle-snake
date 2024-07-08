// Welcome to
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
//
// This file can be a nice home for your Battlesnake logic and helper functions.
//
// To get you started we've included code to prevent your Battlesnake from moving backwards.
// For more info see docs.battlesnake.com

use log::{info, warn};
use rand::seq::SliceRandom;
use rocket::form::validate::Contains;
use serde_json::{json, Value};
use std::{cmp::min, collections::HashMap};

use crate::{Battlesnake, Board, Coord, Dir, Game};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    let mut color = String::new();
    if let Ok(port) = std::env::var("PORT") {
        if port == "8001" {
            info!("RUNNING ON PORT 8081");
            color = "#ffffff".to_string();
        }
    } else {
        color = "#ff69b4".to_string();
    }
    info!("INFO");
    json!({
        "apiversion": "1",
        "author": "", // TODO: Your Battlesnake Username
        "color": color,
        "head": "default", // TODO: Choose head
        "tail": "default", // TODO: Choose tail
    })
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

fn merge(snakes: &[Battlesnake]) -> Vec<Coord> {
    let mut all_enemy_snakes: Vec<Coord> = vec![];
    snakes.iter().for_each(|snake| {
        all_enemy_snakes.push(snake.head.clone());
        all_enemy_snakes.extend(snake.body.clone());
    });
    all_enemy_snakes
}

fn get_distance(c1: &Coord, c2: &Coord) -> i32 {
    (c1.x - c2.x).abs() + (c1.y - c2.y).abs()
}

fn get_nearest_food(head: &Coord, foods: &[Coord]) -> Coord {
    let mut min = i32::MAX;
    let mut min_idx = 0;
    let nearest_food = foods
        .iter()
        .enumerate()
        .find(|(idx, food)| -> bool {
            let dist = get_distance(head, food);
            // NOTE: best case
            if dist == 1 {
                return true;
            }
            if dist < min {
                min = dist;
                min_idx = *idx;
            }
            false
        })
        .unwrap_or((min_idx, &foods[min_idx]));
    warn!("NEAREST FOOD: {:?}, DIST: {min}", nearest_food.1);
    nearest_food.1.clone()
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(_game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Value {
    warn!("{:-^30}", "");
    let mut is_move_safe: HashMap<_, _> = vec![
        ("up", true),
        ("down", true),
        ("left", true),
        ("right", true),
    ]
    .into_iter()
    .collect();

    // We've included code to prevent your Battlesnake from moving backwards
    let my_head = &you.body[0]; // Coordinates of your head
    let my_neck = &you.body[1]; // Coordinates of your "neck"

    let board_width = &board.width;
    let board_height = &board.height;

    if my_neck.x < my_head.x {
        // Neck is left of head, don't move left
        is_move_safe.insert("left", false);
    } else if my_neck.x > my_head.x {
        // Neck is right of head, don't move right
        is_move_safe.insert("right", false);
    } else if my_neck.y < my_head.y {
        // Neck is below head, don't move down
        is_move_safe.insert("down", false);
    } else if my_neck.y > my_head.y {
        // Neck is above head, don't move up
        is_move_safe.insert("up", false);
    }

    if my_head.x == 0 {
        warn!("NOT CRASHING IN THE LEFT WALL");
        is_move_safe.insert("left", false);
    }

    if my_head.y == 0 {
        warn!("NOT CRASHING IN THE BOTTOM WALL");
        is_move_safe.insert("down", false);
    }
    if my_head.x == board_width - 1 {
        warn!("NOT CRASHING IN THE RIGHT WALL");
        is_move_safe.insert("right", false);
    }

    if my_head.y as u32 == board_height - 1 {
        warn!("NOT CRASHING IN THE TOP WALL");
        is_move_safe.insert("up", false);
    }

    let my_body = &you.body;

    if my_body.contains(&my_head.go(Dir::LEFT)) {
        warn!("BODY AT LEFT NOT SAFE!");
        is_move_safe.insert("left", false);
    }

    if my_body.contains(&my_head.go(Dir::RIGHT)) {
        warn!("BODY AT RIGHT NOT SAFE!");
        is_move_safe.insert("right", false);
    }

    if my_body.contains(&my_head.go(Dir::UP)) {
        warn!("BODY AT UP NOT SAFE!");
        is_move_safe.insert("up", false);
    }

    if my_body.contains(&my_head.go(Dir::DOWN)) {
        warn!("BODY AT DOWN NOT SAFE!");
        is_move_safe.insert("down", false);
    }

    let opponents = &board.snakes;
    let occupied_squares = merge(opponents);

    if occupied_squares.contains(&my_head.go(Dir::LEFT)) {
        warn!("ENEMY AT LEFT NOT SAFE!");
        is_move_safe.insert("left", false);
    }

    if occupied_squares.contains(&my_head.go(Dir::RIGHT)) {
        warn!("ENEMY AT RIGHT NOT SAFE!");
        is_move_safe.insert("right", false);
    }

    if occupied_squares.contains(&my_head.go(Dir::UP)) {
        warn!("ENEMY AT UP NOT SAFE!");
        is_move_safe.insert("up", false);
    }

    if occupied_squares.contains(&my_head.go(Dir::DOWN)) {
        warn!("ENEMY AT DOWN NOT SAFE!");
        is_move_safe.insert("down", false);
    }

    // Are there any safe moves left?
    let safe_moves = is_move_safe
        .into_iter()
        .filter(|&(_, v)| v)
        .map(|(k, _)| k)
        .collect::<Vec<_>>();
    let foods = &board.food;
    warn!("TURN: {turn}");
    warn!("HEAD: ({},{})", my_head.x, my_head.y);
    warn!("FOODS: {:?}", foods);
    let nearest_food = get_nearest_food(my_head, foods);
    // NOTE: negative means the food is on the right
    let xdiff = my_head.x - nearest_food.x;
    // NOTE: negative means the food is above
    let ydiff = my_head.y - nearest_food.y;
    let min_diff = min(xdiff.abs(), ydiff.abs());
    // NOTE: true if we should favor x
    let move_x = min_diff == xdiff.abs() && xdiff != 0;
    let can_move_x = safe_moves.iter().any(|x| x == &"left" || x == &"right");
    let can_move_y = safe_moves.iter().any(|x| x == &"top" || x == &"down");
    warn!(
        "DETAILS: move_x: {}, xdiff: {}, ydiff: {}, min_diff: {}, can_move_x: {}, can_move_y: {}",
        move_x, xdiff, ydiff, min_diff, can_move_x, can_move_y
    );
    if move_x && can_move_x && xdiff != 0 {
        if xdiff > 0 && safe_moves.contains("left") {
            warn!("MOVING TO LEFT BECAUSE OF FOOD");
            return json!({ "move": "left" });
        }
        if xdiff < 0 && safe_moves.contains("right") {
            warn!("MOVING TO RIGHT BECAUSE OF FOOD");
            return json!({ "move": "right" });
        }
        let only_move = safe_moves
            .iter()
            .find(|x| **x == "left" || **x == "right")
            .unwrap();
        // INFO: don't move x if x moves causes more distance
        let x_move_good = match *only_move {
            "left" => {
                let cur_dist = get_distance(my_head, &nearest_food);
                let next_dist = get_distance(&my_head.go(Dir::LEFT), &nearest_food);
                next_dist < cur_dist
            }
            "right" => {
                let cur_dist = get_distance(my_head, &nearest_food);
                let next_dist = get_distance(&my_head.go(Dir::RIGHT), &nearest_food);
                next_dist < cur_dist
            }
            _ => false,
        };
        if x_move_good || !can_move_y {
            warn!("MOVING IN X, {only_move} COZ ITS GOOD!");
            return json!({ "move": only_move});
        }
        warn!("MOVING IN X IS DUMB! MOVING WITH Y INSTEAD COZ IT'S POSSIBLE!");
        if ydiff > 0 && safe_moves.contains("down") {
            warn!("MOVING TO DOWN BECAUSE OF FOOD");
            return json!({ "move": "down" });
        }
        if ydiff < 0 && safe_moves.contains("up") {
            warn!("MOVING TO UP BECAUSE OF FOOD");
            return json!({ "move": "up" });
        }
    }
    if can_move_y && ydiff == 0 {
        if xdiff > 0 && safe_moves.contains("left") {
            warn!("MOVING TO LEFT BECAUSE OF FOOD");
            return json!({ "move": "left" });
        }
        if xdiff < 0 && safe_moves.contains("right") {
            warn!("MOVING TO RIGHT BECAUSE OF FOOD");
            return json!({ "move": "right" });
        }
    }
    if !move_x && can_move_y && ydiff != 0 {
        if ydiff > 0 && safe_moves.contains("down") {
            warn!("MOVING TO DOWN BECAUSE OF FOOD");
            return json!({ "move": "down" });
        }
        if ydiff < 0 && safe_moves.contains("up") {
            warn!("MOVING TO UP BECAUSE OF FOOD");
            return json!({ "move": "up" });
        }
        let only_move = safe_moves
            .iter()
            .find(|x| **x == "up" || **x == "down")
            .unwrap();

        warn!(
            "MOVING TO {} BECAUSE OF FOOD, BUT NOT THE BEST",
            only_move.to_uppercase()
        );
        return json!({ "move": only_move});
    }

    if !move_x && !can_move_y {
        if xdiff == 0 {
            if ydiff > 0 && safe_moves.contains("down") {
                warn!("MOVING TO DOWN BECAUSE OF FOOD");
                return json!({ "move": "down" });
            }
            if ydiff < 0 && safe_moves.contains("up") {
                warn!("MOVING TO UP BECAUSE OF FOOD");
                return json!({ "move": "up" });
            }
        }
        if xdiff > 0 && safe_moves.contains("left") {
            warn!("MOVING TO LEFT BECAUSE OF FOOD");
            return json!({ "move": "left" });
        }
        if xdiff < 0 && safe_moves.contains("right") {
            warn!("MOVING TO RIGHT BECAUSE OF FOOD");
            return json!({ "move": "right" });
        }
    }

    let chosen = safe_moves.choose(&mut rand::thread_rng());
    match chosen {
        Some(good_move) => {
            warn!("MOVING SAFELY RANDOMLY COZ ALL EVALUATIONS FAILED");
            warn!("DETAILS: move_x: {}, xdiff: {}, ydiff: {}, can_move_x: {}, can_move_y: {}, min_diff: {}",move_x, xdiff, ydiff, can_move_x, can_move_y, min_diff);
            warn!("{:-^30}", "");
            json!({ "move": good_move})
        }
        None => {
            warn!("NO GOOD MOVE LEFT");
            warn!("{:-^30}", "");
            json!({ "move": "down"})
        }
    }
}
