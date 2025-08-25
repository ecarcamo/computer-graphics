// player.rs

use raylib::prelude::*;
use std::f32::consts::PI;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32, // field of view
}

pub fn process_events(player: &mut Player, rl: &RaylibHandle, delta_time: f32) {
    const MOVE_SPEED: f32 = 200.0;
    const ROTATION_SPEED: f32 = PI ;

    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a += ROTATION_SPEED * delta_time;
    }
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a -= ROTATION_SPEED * delta_time;
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        player.pos.x -= MOVE_SPEED * player.a.cos() * delta_time;
        player.pos.y -= MOVE_SPEED * player.a.sin() * delta_time;
    }
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        player.pos.x += MOVE_SPEED * player.a.cos() * delta_time;
        player.pos.y += MOVE_SPEED * player.a.sin() * delta_time;
    }
}
