use macroquad::math::Vec2;

/// Flip (negate) the selected axis.
/// (42, 12) after `flip_x` will be (-42, 12)
pub trait FlipAxis {
    fn flip_x(&mut self);

    fn flip_y(&mut self);
}

impl FlipAxis for Vec2 {
    fn flip_x(&mut self) {
        self.x = -self.x;
    }

    fn flip_y(&mut self) {
        self.y = -self.y;
    }
}
