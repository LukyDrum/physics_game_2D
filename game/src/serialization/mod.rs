mod rigidbody;
mod sph;

use crate::{
    game::Game,
    physics::{rigidbody::RigidBody, sph::Sph},
    serialization::sph::SphSerializedForm,
};
pub use rigidbody::{BodySerializationForm, BodySerializedForm};
use serde_derive::{Deserialize, Serialize};

pub trait SerializationForm {
    type Original;
    type SerializedForm;

    fn to_serialized_form(&self) -> Self::SerializedForm;

    fn from_serialized_form(serialized_form: Self::SerializedForm) -> Self::Original;
}

#[derive(Serialize, Deserialize)]
pub struct GameSerializedForm {
    pub name: String,
    pub description: String,
    pub width: f32,
    pub height: f32,
    pub rb: RbSerializedForm,
    pub sph: SphSerializedForm,
}

#[derive(Serialize, Deserialize)]
pub struct RbSerializedForm {
    pub bodies: Vec<BodySerializedForm>,
}

impl SerializationForm for Game {
    type Original = Game;

    type SerializedForm = GameSerializedForm;

    fn to_serialized_form(&self) -> Self::SerializedForm {
        let width = self.gameview_width;
        let height = self.gameview_height;
        let name = self.name.clone();
        let description = self
            .description
            .iter()
            .fold(String::new(), |acc, s| acc + "\n" + s);

        let sph = self.fluid_system.to_serialized_form();

        let bodies = self
            .bodies
            .iter()
            .map(|body| body.to_serialized_form())
            .collect();

        GameSerializedForm {
            name,
            description,
            width,
            height,
            sph,
            rb: RbSerializedForm { bodies },
        }
    }

    fn from_serialized_form(serialized_form: Self::SerializedForm) -> Self::Original {
        let GameSerializedForm {
            name,
            description,
            width,
            height,
            sph,
            rb,
        } = serialized_form;

        let sph = Sph::from_serialized_form(sph);
        let bodies = rb
            .bodies
            .into_iter()
            .map(RigidBody::from_serialized_form)
            .collect();

        let mut game = Game::new(width as usize, height as usize);
        game.fluid_system = sph;
        game.bodies = bodies;
        game.name = name;
        game.set_description(description);

        game
    }
}
