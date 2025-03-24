use std::collections::LinkedList;

use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator,
    IntoParallelRefMutIterator, ParallelIterator,
};

use crate::{game::GameBody, math::Vector2};

use super::{BodyBehaviour, BodyCollisionData};

struct BodyBodyCollision {
    index_a: usize,
    index_b: usize,
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
        Self::resolve_collisions(bodies, collisions);
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
                        index_a,
                        index_b,
                        collision_data,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn resolve_collisions(
        bodies: &mut Vec<Box<dyn GameBody>>,
        collisions: LinkedList<BodyBodyCollision>,
    ) {
        let resolutions: LinkedList<CollisionResolution> = collisions
            .par_iter()
            .filter_map(|collision| {
                let body_a = &bodies[collision.index_a];
                let body_b = &bodies[collision.index_b];

                let a_is_dynamic = body_a.state().behaviour == BodyBehaviour::Dynamic;
                let b_is_dynamic = body_b.state().behaviour == BodyBehaviour::Dynamic;

                let penetration = collision.collision_data.penetration;
                let normal = collision.collision_data.normal;

                match (a_is_dynamic, b_is_dynamic) {
                    (true, true) => Some(CollisionResolution {
                        index_a: collision.index_a,
                        index_b: collision.index_b,
                        offset_a: normal * -penetration * 0.5,
                        offset_b: normal * penetration * 0.5,
                    }),
                    (true, false) => Some(CollisionResolution {
                        index_a: collision.index_a,
                        index_b: collision.index_b,
                        offset_a: normal * -penetration,
                        offset_b: Vector2::zero(),
                    }),
                    (false, true) => Some(CollisionResolution {
                        index_a: collision.index_a,
                        index_b: collision.index_b,
                        offset_a: Vector2::zero(),
                        offset_b: normal * penetration,
                    }),
                    (false, false) => None,
                }
            })
            .collect();

        for res in resolutions {
            let CollisionResolution {
                index_a,
                index_b,
                offset_a,
                offset_b,
            } = res;

            bodies[index_a].state_mut().position += offset_a;
            bodies[index_b].state_mut().position += offset_b;
        }
    }
}

struct CollisionResolution {
    index_a: usize,
    index_b: usize,
    offset_a: Vector2<f32>,
    offset_b: Vector2<f32>,
}
