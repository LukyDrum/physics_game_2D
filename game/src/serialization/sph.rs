use crate::{math::Vector2, physics::sph::{Particle, Sph}, rendering::Color};
use serde::{Serialize, Deserialize};

use super::SerializationForm;


#[derive(Serialize, Deserialize)]
pub struct SphSerializedForm {
    pub particles: Vec<ParticleSerializedForm>,
    pub width: f32,
    pub height: f32,
}

impl SerializationForm for Sph {
    type Original = Sph;

    type SerializedForm = SphSerializedForm;

    fn to_serialized_form(&self) -> Self::SerializedForm {
        let ser_form_particles: Vec<ParticleSerializedForm> = self
        .particles
        .iter()
        .map(|p| p.to_serialized_form())
        .collect();

        SphSerializedForm {
            particles: ser_form_particles,
            width: self.lookup.width,
            height: self.lookup.height,
        }
    }

    fn from_serialized_form(serialized_form: Self::SerializedForm) -> Self::Original {
        let SphSerializedForm { particles, width, height } = serialized_form;

        let particles: Vec<Particle> = particles
            .into_iter()
            .map(|ser_p| Particle::from_serialized_form(ser_p))
            .collect();
        
        let mut sph = Sph::new(width, height);
        for p in particles {
            sph.add_particle(p);
        }

        sph
    }
}

#[derive(Serialize, Deserialize)]
pub struct ParticleSerializedForm {
    pub position: Vector2<f32>,
    pub mass: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    /// A multiplier of the force on collision with a rigidbody. This is done to simulate a bigger
    /// ammount of fluid hitting the object instead of only a few particles.
    pub body_collision_force_multiplier: f32,
    pub color: Color,
}

impl SerializationForm for Particle {
    type Original = Particle;

    type SerializedForm = ParticleSerializedForm;

    fn to_serialized_form(&self) -> Self::SerializedForm {
        let Particle {
            position,
            mass,
            target_density,
            pressure_multiplier,
            body_collision_force_multiplier,
            color,
            ..
        } = *self;

        ParticleSerializedForm {
            position,
            mass,
            target_density,
            pressure_multiplier,
            body_collision_force_multiplier,
            color,
        }
    }

    fn from_serialized_form(serialized_form: Self::SerializedForm) -> Self::Original {
        let ParticleSerializedForm {
            position,
            mass,
            target_density,
            pressure_multiplier,
            body_collision_force_multiplier,
            color,
        } = serialized_form;

        Particle {
            position,
            mass,
            target_density,
            pressure_multiplier,
            body_collision_force_multiplier,
            color,
            ..Default::default()
        }
    }
} 
