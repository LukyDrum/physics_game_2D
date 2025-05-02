mod rigidbody;
mod sph;

use crate::{
    game::{Game, GameBody},
    physics::{rigidbody::Polygon, sph::Sph},
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
    pub width: f32,
    pub height: f32,
    pub sph: SphSerializedForm,
    pub rb: RbSerializedForm,
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

        let sph = self.fluid_system.to_serialized_form();

        let bodies = self
            .bodies
            .iter()
            .map(|body| body.to_serialized_form())
            .collect();

        GameSerializedForm {
            width,
            height,
            sph,
            rb: RbSerializedForm { bodies },
        }
    }

    fn from_serialized_form(serialized_form: Self::SerializedForm) -> Self::Original {
        let GameSerializedForm {
            width,
            height,
            sph,
            rb,
        } = serialized_form;

        let sph = Sph::from_serialized_form(sph);
        let bodies = rb
            .bodies
            .into_iter()
            .map(|ser_body| {
                let body: Box<dyn GameBody> = match &ser_body {
                    BodySerializedForm::Polygon(_) => {
                        Box::new(Polygon::from_serialized_form(ser_body))
                    }
                };
                body
            })
            .collect();

        let mut game = Game::new(width as usize, height as usize);
        game.fluid_system = sph;
        game.bodies = bodies;

        game
    }
}
