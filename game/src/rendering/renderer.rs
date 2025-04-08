use crate::Sph;

/// Structs that implement this trait are used for rendering to the game screen.
/// They need to be setup in each iteration and then can draw to screen in their own style.
pub trait Renderer {
    /// Takes references to the simulation systems and setups its internal state to be able to draw
    /// the next frame.
    fn setup(&mut self, sph: &Sph);

    /// Draws to the screen.
    fn draw(&self);
}
