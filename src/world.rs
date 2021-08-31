use nalgebra as na;

/// positions are from 0.0 to 1.0, where zero is left/top
/// vectors are
pub struct World {
    pub ball_pos: na::Point2<f32>,
    pub ball_vector: na::Vector2<f32>,
}

pub fn new_world() -> World {
    World {
        ball_pos: [0.5, 0.5].into(),
        ball_vector: [0.004, 0.001].into(),
    }
}

pub fn render_world(world: &World) {
    use macroquad::prelude::*;

    draw_rectangle(
        screen_width() * world.ball_pos[0],
        screen_height() * world.ball_pos[1],
        10.0,
        10.0,
        RED,
    )
}
