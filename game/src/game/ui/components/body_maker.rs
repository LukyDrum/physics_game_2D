use macroquad::shapes::draw_rectangle;
use macroquad::text::draw_text;
use macroquad::ui::root_ui;
use macroquad::ui::widgets::Checkbox;

use crate::game::{draw_slider, FONT_SIZE_SMALL};
use crate::physics::rigidbody::{
    BodyBehaviour, DEFAULT_DYNAMIC_FRICTION, DEFAULT_ELASTICITY, DEFAULT_STATIC_FRICTION,
};
use crate::utility::AsMq;
use crate::{
    game::UIComponent,
    math::{v2, Vector2},
    rendering::Color,
};

use super::{ColorPicker, GAP, SLIDER_HEIGHT, SLIDER_LENGTH};

const MIN_SIZE: f32 = 5.0;
const DEFAULT_MAX_SIZE: f32 = 500.0;
const MIN_MASS: f32 = 500.0;
const MAX_MASS: f32 = 50_000.0;
const MIN_ORIENTATION: f32 = 0.0;
const MAX_ORIENTATION: f32 = 360.0;

const TUTORIAL_LINES: [&str; 3] = [
    "[Left MB] - Drag rigidbodies",
    "[Right MB] - Spawn new rigidbody",
    "[Middle MB] - Delete rigidbody under cursor",
];

pub struct BodyMaker {
    width: f32,
    height: f32,
    pub mass: f32,
    pub orientation: f32,
    pub lock_rotation: bool,
    pub behaviour: BodyBehaviour,

    pub elasticity: f32,
    pub static_friction: f32,
    pub dynamic_friction: f32,

    max_size: f32,
    changed: bool,

    color_picker: ColorPicker,
}

impl Default for BodyMaker {
    fn default() -> Self {
        BodyMaker {
            width: 30.0,
            height: 30.0,
            mass: 5000.0,
            orientation: 0.0,
            lock_rotation: false,
            behaviour: BodyBehaviour::Dynamic,

            elasticity: DEFAULT_ELASTICITY,
            static_friction: DEFAULT_STATIC_FRICTION,
            dynamic_friction: DEFAULT_DYNAMIC_FRICTION,

            max_size: DEFAULT_MAX_SIZE,
            changed: false,

            color_picker: ColorPicker::new(Color::rgb(0, 0, 0)),
        }
    }
}

impl UIComponent for BodyMaker {
    fn draw(&mut self, offset: Vector2<f32>) {
        let BodyMaker {
            width: old_width,
            height: old_height,
            mass: old_mass,
            orientation: old_orientation,
            lock_rotation: old_lock_rotation,
            behaviour: old_behaviour,
            elasticity: old_elasticity,
            static_friction: old_static_friction,
            dynamic_friction: old_dynamic_friction,
            ..
        } = *self;

        let mut offset = offset;
        for line in TUTORIAL_LINES {
            draw_text(
                line,
                offset.x,
                offset.y,
                FONT_SIZE_SMALL,
                Color::rgb(0, 0, 0).as_mq(),
            );
            offset += v2!(0.0, FONT_SIZE_SMALL + 10.0);
        }

        draw_slider(
            offset,
            "Width [cm]",
            370.0,
            &mut self.width,
            MIN_SIZE..self.max_size,
        );

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + GAP);
        draw_slider(
            offset,
            "Height [cm]",
            SLIDER_LENGTH,
            &mut self.height,
            MIN_SIZE..self.max_size,
        );

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + GAP);
        draw_slider(
            offset,
            "Orientation [degrees]",
            SLIDER_LENGTH,
            &mut self.orientation,
            MIN_ORIENTATION..MAX_ORIENTATION,
        );
        let side_offset = offset + v2!(450.0, 0.0);
        Checkbox::new(68)
            .pos(side_offset.as_mq())
            .label("Lock rotation?")
            .size(v2!(SLIDER_HEIGHT, SLIDER_HEIGHT).as_mq())
            .ui(&mut root_ui(), &mut self.lock_rotation);

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + GAP);
        draw_slider(
            offset,
            "Mass [g]",
            SLIDER_LENGTH,
            &mut self.mass,
            MIN_MASS..MAX_MASS,
        );

        let side_offset = offset + v2!(400.0, 0.0);
        let mut is_static = self.behaviour == BodyBehaviour::Static;
        Checkbox::new(69)
            .pos(side_offset.as_mq())
            .label("Is static?")
            .size(v2!(SLIDER_HEIGHT, SLIDER_HEIGHT).as_mq())
            .ui(&mut root_ui(), &mut is_static);
        self.behaviour = if is_static {
            let x = side_offset.x + 0.25 * SLIDER_HEIGHT - 19.0;
            let y = side_offset.y + 0.25 * SLIDER_HEIGHT;
            let wh = SLIDER_HEIGHT * 0.5;
            draw_rectangle(x, y, wh, wh, Color::rgb(255, 255, 255).as_mq());

            BodyBehaviour::Static
        } else {
            BodyBehaviour::Dynamic
        };

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + GAP);
        draw_slider(
            offset,
            "Elasticity",
            SLIDER_LENGTH,
            &mut self.elasticity,
            0.05..0.95,
        );

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + GAP);
        draw_slider(
            offset,
            "Static friction",
            SLIDER_LENGTH,
            &mut self.static_friction,
            0.05..0.95,
        );

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + GAP);
        draw_slider(
            offset,
            "Dynamic friction",
            SLIDER_LENGTH,
            &mut self.dynamic_friction,
            0.05..0.95,
        );

        let old_color = self.color_picker.color();
        self.color_picker
            .draw(offset + v2!(0.0, SLIDER_HEIGHT + 25.0));

        self.changed = self.width != old_width
            || self.height != old_height
            || self.mass != old_mass
            || self.orientation != old_orientation
            || self.lock_rotation != old_lock_rotation
            || old_color != self.color_picker.color()
            || self.behaviour != old_behaviour
            || self.elasticity != old_elasticity
            || self.static_friction != old_static_friction
            || self.dynamic_friction != old_dynamic_friction;
    }
}

impl BodyMaker {
    pub fn color(&self) -> Color {
        self.color_picker.color()
    }

    pub fn size(&self) -> Vector2<f32> {
        v2!(self.width, self.height)
    }

    pub fn set_max_size(&mut self, new_max: f32) {
        self.max_size = new_max;
    }

    pub fn changed(&self) -> bool {
        self.changed
    }
}
