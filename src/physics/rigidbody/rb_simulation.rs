use crate::game::GameBody;

pub struct RbSimulator {}

impl RbSimulator {
    pub fn step(&self, bodies: &mut Vec<Box<dyn GameBody>>) {}
}
