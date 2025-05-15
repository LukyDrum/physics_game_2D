use crate::game::ui::HEADER_TOOL_GAP;
use crate::game::{red_button_skin, UIComponent, FONT_SIZE_LARGE};
use crate::math::v2;
use crate::math::Vector2;
use crate::rendering::Color;
use crate::utility::AsMq;

use macroquad::text::draw_text;
use macroquad::ui::root_ui;
use macroquad::ui::widgets::Button;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuickAction {
    Nothing,
    Quit,
    Restart,
    TogglePause,
}

impl Default for QuickAction {
    fn default() -> Self {
        QuickAction::Nothing
    }
}

#[derive(Default)]
pub struct QuickMenu {
    pub action: QuickAction,
}

impl UIComponent for QuickMenu {
    fn draw(&mut self, offset: Vector2<f32>) {
        draw_text(
            "Quick Menu",
            offset.x,
            offset.y,
            FONT_SIZE_LARGE,
            Color::rgb(0, 0, 0).as_mq(),
        );
        let offset = offset + v2!(0.0, HEADER_TOOL_GAP);

        let red_skin = red_button_skin();
        let default_skin = root_ui().default_skin();

        let items = [
            ("Restart", QuickAction::Restart, &red_skin),
            ("Quit", QuickAction::Quit, &red_skin),
            ("(Un)Pause", QuickAction::TogglePause, &default_skin),
        ];

        for (row_index, item) in items.iter().enumerate() {
            let position = offset + v2!(0.0, 50.0) * row_index as f32;

            root_ui().push_skin(item.2);
            if Button::new(item.0)
                .size(v2!(130.0, 30.0).as_mq())
                .position(position.as_mq())
                .ui(&mut root_ui())
                && self.action != item.1
            {
                self.action = item.1;
                root_ui().pop_skin();
                return;
            }
            root_ui().pop_skin();
        }

        self.action = QuickAction::Nothing;
    }
}
