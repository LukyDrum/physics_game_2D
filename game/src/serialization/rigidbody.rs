use crate::rendering::Color;
use crate::serialization::SerializationForm;
use crate::{
    math::Vector2,
    physics::rigidbody::{BodyBehaviour, BodyState, Polygon},
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
}

#[derive(Serialize, Deserialize)]
pub struct BodyStateSerializedForm {
    pub position: Vector2<f32>,
    pub orientation: f32,

    pub behaviour: BodyBehaviour,
    pub mass: f32,
    pub moment_of_inertia: f32,
    pub elasticity: f32,
    pub static_friction: f32,
    pub dynamic_friction: f32,

    pub color: Color,
}

impl From<BodyState> for BodyStateSerializedForm {
    fn from(body_state: BodyState) -> BodyStateSerializedForm {
        let BodyState {
            position,
            orientation,
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

impl BodySerializationForm for Polygon {
    fn to_serialized_form(&self) -> BodySerializedForm {
        let points = self.points.clone();
        let ser_state = self.state.clone().into();

        BodySerializedForm::Polygon(PolygonSerializedForm {
            state: ser_state,
            points,
        })
    }

    #[allow(irrefutable_let_patterns)]
    fn from_serialized_form(serialized_form: BodySerializedForm) -> Self {
        let BodySerializedForm::Polygon(serialized_form) = serialized_form else {
            panic!("Passed in invalid serialized form!");
        };

        let points = serialized_form.points;
        let state: BodyState = serialized_form.state.into();

        let mut polygon = Polygon::new(state.position, points, state.behaviour);
        polygon.state = state;

        polygon
    }
}
