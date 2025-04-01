use core::{f32, panic};
use std::{
    collections::LinkedList,
    ops::{Add, Mul},
};

use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use super::{BodyBehaviour, BodyCollisionData};
use crate::{game::GameBody, math::Vector2};

/// Holds `BodyCollisionData` along with indexes of what two bodies collided.
#[derive(Clone)]
struct BodyBodyCollision {
    index_a: usize,
    index_b: usize,
    collision_data: BodyCollisionData,
}

pub enum SharedPropertySelection {
    Multiply,
    Average,
    Min,
    Max,
}

impl SharedPropertySelection {
    pub fn select<T>(&self, a: T, b: T) -> T
    where
        T: PartialOrd + Mul<Output = T> + Mul<f32, Output = T> + Add<Output = T>,
    {
        match self {
            Self::Multiply => a * b,
            Self::Average => (a + b) * 0.5,
            Self::Min => {
                if a < b {
                    a
                } else {
                    b
                }
            }
            Self::Max => {
                if a > b {
                    a
                } else {
                    b
                }
            }
        }
    }
}

pub struct RbSimulator {
    pub gravity: Vector2<f32>,
    pub elasticity_selection: SharedPropertySelection,
    pub friction_selection: SharedPropertySelection,

    pub current_time_step: f32,
}

impl RbSimulator {
    /// Only correct the position of the body by this much percent
    const CORRECTION_STABILIZER: f32 = 0.8;
    /// For stability, we tolerate some very small penetration
    const PENETRATION_TOLERANCE: f32 = 1.0;

    /// Max impulse magnitude for applying static friction
    const STATIC_FRICTION_LIMIT: f32 = 10.0;

    pub fn new(gravity: Vector2<f32>) -> Self {
        RbSimulator {
            gravity,
            elasticity_selection: SharedPropertySelection::Average,
            friction_selection: SharedPropertySelection::Average,

            current_time_step: 0.0,
        }
    }

    pub fn step(&mut self, bodies: &mut Vec<Box<dyn GameBody>>, time_step: f32) {
        // Set timestep for this step
        self.current_time_step = time_step;

        // Apply and move bodies by gravity
        self.apply_gravity(bodies, time_step);
        Self::move_bodies_by_velocity(bodies, time_step);

        // Update inner values to reflect the change due to gravity.
        Self::update_inner_values(bodies);

        let collisions = Self::check_collisions(bodies);
        self.resolve_collisions(bodies, collisions);

        Self::update_inner_values(bodies);
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
        &self,
        bodies: &mut Vec<Box<dyn GameBody>>,
        collisions: LinkedList<BodyBodyCollision>,
    ) {
        for coll in &collisions {
            let BodyBodyCollision {
                index_a,
                index_b,
                collision_data,
            } = coll.clone();

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
            let mass_a = bodies[index_a].state().mass();
            let velocity_a = bodies[index_a].state().velocity;
            let angular_velocity_a = bodies[index_a].state().angular_velocity;
            let inertia_a = bodies[index_a].state().moment_of_inertia();
            let inv_inertia_a = inverse_value(inertia_a);
            let center_a = bodies[index_a].center_of_mass();
            // Values of B
            let mass_b = bodies[index_b].state().mass();
            let velocity_b = bodies[index_b].state().velocity;
            let angular_velocity_b = bodies[index_b].state().angular_velocity;
            let inertia_b = bodies[index_b].state().moment_of_inertia();
            let inv_inertia_b = inverse_value(inertia_b);
            let center_b = bodies[index_b].center_of_mass();

            // Shared properties
            let shared_elasticity = {
                let elasticity_a = bodies[index_a].state().elasticity;
                let elasticity_b = bodies[index_b].state().elasticity;
                self.elasticity_selection.select(elasticity_a, elasticity_b)
            };
            let shared_friction = {
                let friction_a = bodies[index_a].state().dynamic_friction;
                let friction_b = bodies[index_b].state().dynamic_friction;
                self.friction_selection.select(friction_a, friction_b)
            };

            let inv_masses = inverse_value(mass_a) + inverse_value(mass_b);
            // Apply impulse for each collision point weighted by the number of collision points
            let multiplier = 1.0 / collision_points.len() as f32;
            for coll_point in collision_points {
                let radius_a = coll_point - center_a;
                let radius_b = coll_point - center_b;

                // Relative velocity of the contact point from both bodies
                let relative_velocity = (velocity_a
                    + scalar_vector_cross(angular_velocity_a, radius_a))
                    - (velocity_b + scalar_vector_cross(angular_velocity_b, radius_b));

                // Formula for calculation of the effective mass in direction. The bottom term in
                // the impulse calculation.
                let effective_mass_formula = |dir: Vector2<f32>| {
                    let inertia_term_a =
                        scalar_vector_cross(radius_a.cross(dir), radius_a) * inv_inertia_a;
                    let inertia_term_b =
                        scalar_vector_cross(radius_b.cross(dir), radius_b) * inv_inertia_b;

                    inv_masses + (inertia_term_a + inertia_term_b).dot(dir)
                };

                // Normal impulse
                let top_term = -(1.0 + shared_elasticity) * relative_velocity.dot(normal);
                let impulse_normal = top_term / effective_mass_formula(normal) * multiplier;

                // Tangent impulse - friction
                let tangent = normal.normal();
                let mut impulse_tangent =
                    relative_velocity.dot(tangent) / effective_mass_formula(tangent) * multiplier;
                if impulse_tangent.abs() > Self::STATIC_FRICTION_LIMIT {
                    impulse_tangent *= shared_friction;
                }

                // Add impulses to both bodies
                if a_is_dynamic {
                    let state = bodies[index_a].state_mut();
                    // Apply normal impulse
                    state.velocity += normal * (impulse_normal / mass_a);
                    state.angular_velocity +=
                        radius_a.cross(normal * impulse_normal) * inv_inertia_a;

                    // Apply tangent impulse - friction
                    state.velocity -= tangent * (impulse_tangent / mass_a);
                    state.angular_velocity -=
                        radius_a.cross(tangent * impulse_tangent) * inv_inertia_a;
                }
                if b_is_dynamic {
                    let state = bodies[index_b].state_mut();
                    // Apply normal impulse
                    state.velocity -= normal * (impulse_normal / mass_b);
                    state.angular_velocity -=
                        radius_b.cross(normal * impulse_normal) * inv_inertia_b;

                    // Apply tangent impulse - friction
                    state.velocity += tangent * (impulse_tangent / mass_b);
                    state.angular_velocity +=
                        radius_b.cross(tangent * impulse_tangent) * inv_inertia_b;
                }
            }

            // Offset the bodies positions by the penetration
            let (a_percent, b_percent) = match (a_is_dynamic, b_is_dynamic) {
                    (true, true) => (0.5 * (mass_a / (mass_a + mass_b)), 0.5 * (mass_b / (mass_a + mass_b))),
                    (true, false) => (1.0, 0.0),
                    (false, true) => (0.0, 1.0),
                    // This case should not be possible
                    (false, false) => panic!("This case should not be possible as the loop should have skipped to next iteration."),
            };
            let correction =
                (penetration - Self::PENETRATION_TOLERANCE).max(0.0) * Self::CORRECTION_STABILIZER;
            if a_is_dynamic {
                let state = bodies[index_a].state_mut();
                state.position -= normal * correction * a_percent;
            }
            if b_is_dynamic {
                let state = bodies[index_b].state_mut();
                state.position += normal * correction * b_percent;
            }
        }
    }
}

/// Creates an inverse of the `value`, that is:
///   - `1.0 / value` if `value != +-INF`
///   - `0.0` if `value == INF`
fn inverse_value(value: f32) -> f32 {
    if value == f32::INFINITY || value == f32::NEG_INFINITY {
        0.0
    } else {
        1.0 / value
    }
}

/// As if: `(0, 0, scalar) x (v.x, v.y, 0)`
fn scalar_vector_cross(scalar: f32, vector: Vector2<f32>) -> Vector2<f32> {
    let x = -scalar * vector.y;
    let y = scalar * vector.x;
    Vector2::new(x, y)
}
