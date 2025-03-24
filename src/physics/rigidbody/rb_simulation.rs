use std::collections::LinkedList;

use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator,
    IntoParallelRefMutIterator, ParallelIterator,
};

use crate::{game::GameBody, math::Vector2};

use super::{BodyBehaviour, BodyCollisionData};

struct BodyBodyCollision {
    body_a_index: usize,
    body_b_index: usize,
    collision_data: BodyCollisionData,
}

pub struct RbSimulator {
    pub gravity: Vector2<f32>,
}

impl RbSimulator {
    pub fn new(gravity: Vector2<f32>) -> Self {
        RbSimulator { gravity }
    }

    pub fn step(&mut self, bodies: &mut Vec<Box<dyn GameBody>>, time_step: f32) {
        // Apply and move bodies by gravity
        self.apply_gravity(bodies, time_step);
        Self::move_bodies_by_velocity(bodies, time_step);

        // Update inner values to reflect the change due to gravity.
        // Static bodies do not need to be updated as they did not move in any way.
        Self::update_inner_values(bodies);

        let collisions = Self::check_collisions(bodies);
    }

    /// Update the inner stored values of each body, such as global vertices or lines.
    fn update_inner_values(bodies: &mut Vec<Box<dyn GameBody>>) {
        bodies
            .par_iter_mut()
            .for_each(|body| body.update_inner_values());
    }

    /// Applies gravity force to bodies with behaviour set to `BodyBehaviour::Dynamic`.
    fn apply_gravity(&self, bodies: &mut Vec<Box<dyn GameBody>>, time_step: f32) {
        bodies
            .par_iter_mut()
            .filter(|body| body.state().behaviour == BodyBehaviour::Dynamic)
            .for_each(|body| {
                let state = body.state_mut();
                state.add_force(self.gravity * state.mass);

                state.apply_accumulated_force(time_step);
            });
    }

    fn move_bodies_by_velocity(bodies: &mut Vec<Box<dyn GameBody>>, time_step: f32) {
        bodies
            .par_iter_mut()
            .for_each(|body| body.state_mut().move_by_velocity(time_step));
    }

    /// Checks for possible collisions and returns a `LinkedList` of `BodyBodyCollision` where each
    /// record represents a collison between 2 bodies.
    fn check_collisions(bodies: &Vec<Box<dyn GameBody>>) -> LinkedList<BodyBodyCollision> {
        let mut index_pairs = LinkedList::new();
        for i in 1..bodies.len() {
            for j in 0..i {
                index_pairs.push_back((i, j));
            }
        }

        index_pairs
            .into_par_iter()
            .filter_map(|(index_a, index_b)| {
                if let Some(collision_data) =
                    bodies[index_a].check_collision_against(&bodies[index_b])
                {
                    Some(BodyBodyCollision {
                        body_a_index: index_a,
                        body_b_index: index_b,
                        collision_data,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
