use macroquad::math::Vec2;

/// Flip (negate) the selected axis.
/// (42, 12) after `flip_x` will be (-42, 12)
pub trait FlipAxis {
    fn flip_x(&mut self);

    fn flip_y(&mut self);
}

impl FlipAxis for Vec2 {
    fn flip_x(&mut self) {
        if self.x == 0.0 {
            return;
        }
        self.x = -self.x;
    }

    fn flip_y(&mut self) {
        if self.y == 0.0 {
            return;
        }
        self.y = -self.y;
    }
}
