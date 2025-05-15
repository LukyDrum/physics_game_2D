use crate::physics::rigidbody::{RigidBody, SharedProperty};
use crate::rendering::Color;
use crate::{
    math::Vector2,
    physics::rigidbody::{BodyBehaviour, BodyState},
};
use serde_derive::{Deserialize, Serialize};

pub trait BodySerializationForm {
    fn to_serialized_form(&self) -> BodySerializedForm;

    fn from_serialized_form(ser_body: BodySerializedForm) -> Self
    where
        Self: Sized;
}

#[derive(Serialize, Deserialize)]
pub enum BodySerializedForm {
    Polygon(PolygonSerializedForm),
    Circle(CircleSerializedForm),
}

#[derive(Serialize, Deserialize)]
pub struct BodyStateSerializedForm {
    pub position: Vector2<f32>,
    pub orientation: f32,
    #[serde(default)]
    pub lock_rotation: bool,

    pub behaviour: BodyBehaviour,
    pub mass: f32,
    pub moment_of_inertia: f32,
    pub elasticity: SharedProperty<f32>,
    pub static_friction: SharedProperty<f32>,
    pub dynamic_friction: SharedProperty<f32>,

    pub color: Color,
}

impl From<BodyState> for BodyStateSerializedForm {
    fn from(body_state: BodyState) -> BodyStateSerializedForm {
        let BodyState {
            position,
            orientation,
            lock_rotation,
            behaviour,
            mass,
            moment_of_inertia,
            elasticity,
            static_friction,
            dynamic_friction,
            color,
            ..
        } = body_state;

        BodyStateSerializedForm {
            position,
            orientation,
            lock_rotation,
            behaviour,
            mass,
            moment_of_inertia,
            elasticity,
            static_friction,
            dynamic_friction,
            color,
        }
    }
}

impl From<BodyStateSerializedForm> for BodyState {
    fn from(serialized_from: BodyStateSerializedForm) -> BodyState {
        let BodyStateSerializedForm {
            position,
            orientation,
            lock_rotation,
            behaviour,
            mass,
            moment_of_inertia,
            elasticity,
            static_friction,
            dynamic_friction,
            color,
        } = serialized_from;

        BodyState {
            position,
            orientation,
            lock_rotation,
            behaviour,
            mass,
            moment_of_inertia,
            elasticity,
            static_friction,
            dynamic_friction,
            color,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PolygonSerializedForm {
    pub state: BodyStateSerializedForm,
    pub points: Vec<Vector2<f32>>,
}

#[derive(Serialize, Deserialize)]
pub struct CircleSerializedForm {
    pub state: BodyStateSerializedForm,
    pub radius: f32,
}

impl BodySerializationForm for RigidBody {
    fn to_serialized_form(&self) -> BodySerializedForm {
        match self {
            Self::Polygon(inner) => {
                let points = inner.points.clone();
                let ser_state = self.state().clone().into();

                BodySerializedForm::Polygon(PolygonSerializedForm {
                    state: ser_state,
                    points,
                })
            }
            Self::Circle(inner) => BodySerializedForm::Circle(CircleSerializedForm {
                state: self.state().clone().into(),
                radius: inner.radius,
            }),
        }
    }

    #[allow(irrefutable_let_patterns)]
    fn from_serialized_form(serialized_form: BodySerializedForm) -> Self {
        match serialized_form {
            BodySerializedForm::Polygon(serialized_form) => {
                let points = serialized_form.points;
                let state: BodyState = serialized_form.state.into();

                let mut polygon = RigidBody::new_polygon(state.position, points, state.behaviour);
                *polygon.state_mut() = state;

                polygon
            }
            BodySerializedForm::Circle(serialized_form) => {
                let radius = serialized_form.radius;
                let state: BodyState = serialized_form.state.into();

                let mut circle = RigidBody::new_circle(state.position, radius, state.behaviour);
                *circle.state_mut() = state;

                circle
            }
        }
    }
}
