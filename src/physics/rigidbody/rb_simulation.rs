use core::panic;
use std::collections::LinkedList;

use num_traits::{float::FloatCore, Float};
use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{game::GameBody, math::Vector2};

use super::{BodyBehaviour, BodyCollisionData};

/// Holds `BodyCollisionData` along with indexes of what two bodies collided.
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

                state.apply_accumulated_forces(time_step);
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
                // Skip over pairs where both bodies are `Static`
                if bodies[index_a].state().behaviour == BodyBehaviour::Static
                    && bodies[index_b].state().behaviour == BodyBehaviour::Static
                {
                    None
                } else if let Some(collision_data) =
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

    /// Applies appropriate forces to bodies in order to resolve all collisions.
    fn resolve_collisions(
        bodies: &mut Vec<Box<dyn GameBody>>,
        collisions: LinkedList<BodyBodyCollision>,
    ) {
        for coll in collisions {
            let BodyBodyCollision {
                index_a,
                index_b,
                collision_data,
            } = coll;

            let a_is_dynamic = bodies[index_a].state().behaviour == BodyBehaviour::Dynamic;
            let b_is_dynamic = bodies[index_b].state().behaviour == BodyBehaviour::Dynamic;

            // If both bodies are `Static`, then just skip them - no resolution here
            if !a_is_dynamic && !b_is_dynamic {
                continue;
            }

            let BodyCollisionData {
                normal,
                penetration,
                collision_points,
            } = collision_data;

            // Calculate needed values
            // Values of A
            let mass_a = bodies[index_a].state().mass;
            let velocity_a = bodies[index_a].state().velocity;
            let angular_velocity_a = bodies[index_a].state().angular_velocity;
            let inertia_a = bodies[index_a].moment_of_inertia();
            let center_a = bodies[index_a].center_of_mass();
            // Values of B
            let mass_b = bodies[index_b].state().mass;
            let velocity_b = bodies[index_b].state().velocity;
            let angular_velocity_b = bodies[index_b].state().angular_velocity;
            let inertia_b = bodies[index_b].moment_of_inertia();
            let center_b = bodies[index_b].center_of_mass();

            // Apply impulse for each collision point weighted by the number of collision points
            let multiplier = 1.0 / collision_points.len() as f32;
            for coll_point in collision_points {
                let radius_a = coll_point - center_a;
                let radius_b = coll_point - center_b;

                let relative_velocity = (velocity_b + radius_b * angular_velocity_b)
                    - (velocity_a + radius_a * angular_velocity_a);

                let inv_masses = 1.0 / mass_a + 1.0 / mass_b;
                let angular_term_fn = |v: Vector2<f32>| {
                    let inv_inertia_a = 1.0 / inertia_a;
                    let inv_inertia_b = 1.0 / inertia_b;
                    let cross_a = scalar_vector_cross(radius_a.cross(v), radius_a);
                    let cross_b = scalar_vector_cross(radius_b.cross(v), radius_b);

                    (cross_a * inv_inertia_a + cross_b * inv_inertia_b).dot(v)
                };
                let elasticity = 0.2;
                let impulse_normal = {
                    let numerator = relative_velocity.dot(normal) * -(1.0 + elasticity);
                    let denominator = inv_masses + angular_term_fn(normal);

                    (numerator / denominator) * multiplier
                };
                // Friction
                let tangent = normal.normal();
                let impulse_tangent = {
                    let numerator = -relative_velocity.dot(tangent);
                    let denominator = inv_masses + angular_term_fn(tangent);

                    (numerator / denominator) * multiplier
                };

                // Add impulses to both bodies
                if a_is_dynamic {
                    let state = bodies[index_a].state_mut();
                    // Apply normal impulse
                    state.velocity -= normal * (impulse_normal / mass_a);
                    state.angular_velocity -= (impulse_normal / inertia_a) * radius_a.cross(normal);
                    // Apply tangent impulse
                    state.velocity -= tangent * (impulse_tangent / mass_a);
                    state.angular_velocity -= (impulse_tangent / inertia_a) * radius_a.cross(tangent);

                }
                if b_is_dynamic {
                    let state = bodies[index_b].state_mut();
                    // Apply normal impulse
                    state.velocity += normal * (impulse_normal / mass_b);
                    state.angular_velocity -= (impulse_normal / inertia_b) * radius_b.cross(normal);
                    // Apply tangent impulse
                    state.velocity += tangent * (impulse_tangent / mass_b);
                    state.angular_velocity -= (impulse_tangent / inertia_b) * radius_b.cross(tangent);
                }
            }

            // Offset the bodies positions by the penetration
            let (a_pen, b_pen) = match (a_is_dynamic, b_is_dynamic) {
                    (true, true) => (0.5 * (mass_a / (mass_a + mass_b)), 0.5 * (mass_b / (mass_a + mass_b))),
                    (true, false) => (1.0, 0.0),
                    (false, true) => (0.0, 1.0),
                    // This case should not be possible
                    (false, false) => panic!("This case should not be possible as the loop should have skipped to next iteration."),
                };
            if a_is_dynamic {
                let state = bodies[index_a].state_mut();
                state.position -= normal * penetration * a_pen;
            }
            if b_is_dynamic {
                let state = bodies[index_b].state_mut();
                state.position += normal * penetration * b_pen;
            }
        }
    }
}

fn scalar_vector_cross(scalar: f32, vector: Vector2<f32>) -> Vector2<f32> {
    Vector2::new(-scalar * vector.y, scalar * vector.x)
}
