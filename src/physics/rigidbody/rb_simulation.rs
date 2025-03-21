use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{game::GameBody, math::Vector2};

use super::BodyBehaviour;

pub struct RbSimulator {
    pub gravity: Vector2<f32>,
}

impl RbSimulator {
    pub fn new(gravity: Vector2<f32>) -> Self {
        RbSimulator { gravity }
    }
}

impl RbSimulator {
    pub fn step(&mut self, bodies: &mut Vec<Box<dyn GameBody>>, time_step: f32) {
        let mut sim_bodies = bodies
            .par_iter_mut()
            .filter(|body| body.state().behaviour == BodyBehaviour::Dynamic)
            .collect();

        self.apply_gravity(&mut sim_bodies, time_step);
        self.move_bodies_by_velocity(&mut sim_bodies, time_step);
        self.update_inner_values(&mut sim_bodies);
    }

    fn update_inner_values(&self, sim_bodies: &mut Vec<&mut Box<dyn GameBody>>) {
        sim_bodies.par_iter_mut().for_each(|body| body.pre_update());
    }

    fn apply_gravity(&self, sim_bodies: &mut Vec<&mut Box<dyn GameBody>>, time_step: f32) {
        sim_bodies.par_iter_mut().for_each(|body| {
            let state = body.state_mut();
            state.add_force(self.gravity * state.mass);

            state.apply_accumulated_force(time_step);
        });
    }

    fn move_bodies_by_velocity(
        &self,
        sim_bodies: &mut Vec<&mut Box<dyn GameBody>>,
        time_step: f32,
    ) {
        sim_bodies
            .par_iter_mut()
            .for_each(|body| body.state_mut().move_by_velocity(time_step));
    }
}
