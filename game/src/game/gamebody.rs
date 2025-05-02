use crate::{
    physics::rigidbody::{Body, Polygon},
    rendering::Draw, serialization::BodySerializationForm,
};

pub trait GameBody: Body + Draw + BodySerializationForm {}

impl GameBody for Polygon {}
