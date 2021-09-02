use nalgebra as na;

/// positions are from 0.0 to 1.0, where zero is left/top
/// vectors are
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct World {
    pub ball_pos: na::Point2<f32>,
    pub ball_vector: na::Vector2<f32>,
    pub paddle_left: na::Point2<f32>,
    pub paddle_right: na::Point2<f32>,
}

pub fn new_world() -> World {
    World {
        ball_pos: [0.5, 0.5].into(),
        ball_vector: [0.004, 0.001].into(),
        paddle_left: [0.08, 0.5].into(),
        paddle_right: [1.0 - 0.04, 0.5].into(),
    }
}

pub fn render_world(world: &World, color: macroquad::color::Color) {
    use macroquad::prelude::*;

    draw_rectangle(
        screen_width() * world.ball_pos[0] - screen_width() * 0.02,
        screen_height() * world.ball_pos[1] - screen_height() * 0.02,
        screen_width() * 0.02,
        screen_height() * 0.02,
        color,
    );
    draw_rectangle(
        screen_width() * world.paddle_left[0] - screen_width() * 0.02,
        screen_height() * world.paddle_left[1] - screen_height() * 0.1,
        screen_width() * 0.02,
        screen_height() * 0.1,
        color,
    );
    draw_rectangle(
        screen_width() * world.paddle_right[0] - screen_width() * 0.02,
        screen_height() * world.paddle_right[1] - screen_height() * 0.1,
        screen_width() * 0.02,
        screen_height() * 0.1,
        color,
    );
}
