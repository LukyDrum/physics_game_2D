mod rigidbody;
mod sph;

use crate::{game::Game, serialization::sph::SphSerializedForm};
pub use rigidbody::{BodySerializedForm, BodySerializationForm};
use serde::{Serialize, Deserialize};


pub trait SerializationForm {
    type Original;
    type SerializedForm;

    fn to_serialized_form(&self) -> Self::SerializedForm;

    fn from_serialized_form(serialized_form: Self::SerializedForm) -> Self::Original;
}

#[derive(Serialize, Deserialize)]
pub struct GameSerializedForm {
    pub width: f32,
    pub height: f32,
    pub sph: SphSerializedForm,
    pub rb: RbSerializedForm,
}

#[derive(Serialize, Deserialize)]
pub struct RbSerializedForm {
    pub bodies: Vec<BodySerializedForm>
}

impl SerializationForm for Game {
    type Original = Game;

    type SerializedForm = GameSerializedForm;

    fn to_serialized_form(&self) -> Self::SerializedForm {
        let width = self.gameview_width;
        let height = self.gameview_height;

        let sph = self.fluid_system.to_serialized_form();
        
        let bodies = self.bodies.iter().map(|body| body.to_serialized_form()).collect();

        GameSerializedForm {
            width,
            height,
            sph,
            rb: RbSerializedForm { bodies }
        }
    }

    fn from_serialized_form(serialized_form: Self::SerializedForm) -> Self::Original {
        todo!()
    }
}
