use std::ops::RangeInclusive;

use crate::Inspectable;
use bevy::math::Vec2;
use bevy_egui::egui::{self, containers, Rect};
use egui::{Pos2, Sense, Widget};

#[derive(Debug)]
pub struct Vec2dAttributes {
    pub min: Vec2,
    pub max: Vec2,
}
impl Default for Vec2dAttributes {
    fn default() -> Self {
        Vec2dAttributes {
            min: Vec2::new(-100.0, -100.0),
            max: Vec2::new(100.0, 100.0),
        }
    }
}

impl Inspectable for Vec2 {
    type FieldOptions = Vec2dAttributes;

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, options: crate::Options<Self::FieldOptions>) {
        let mut frame = containers::Frame::dark_canvas(&ui.style());
        frame.margin = egui::Vec2::zero();

        frame.show(ui, |ui| {
            let range = options.custom.min..=options.custom.max;
            let widget = PointSelect::new(self, range, 80.0);
            ui.add(widget);
        });
    }
}

struct PointSelect<'a> {
    size: egui::Vec2,
    circle_radius: f32,
    range: RangeInclusive<Vec2>,
    value: &'a mut Vec2,
}
impl<'a> PointSelect<'a> {
    fn new(value: &'a mut Vec2, range: RangeInclusive<Vec2>, size: f32) -> Self {
        PointSelect {
            value,
            range,
            circle_radius: 4.0,
            size: egui::Vec2::new(size, size),
        }
    }

    fn x_range(&self) -> RangeInclusive<f32> {
        self.range.start().x..=self.range.end().x
    }
    fn y_range(&self) -> RangeInclusive<f32> {
        self.range.end().y..=self.range.start().y
    }

    fn value_to_ui_pos(&self, rect: &Rect) -> Pos2 {
        let x = egui::remap_clamp(self.value.x, self.x_range(), rect.x_range());
        let y = egui::remap_clamp(self.value.y, self.y_range(), rect.y_range());
        Pos2::new(x, y)
    }
    fn ui_pos_to_value(&self, rect: &Rect, pos: Pos2) -> Vec2 {
        let x = egui::remap_clamp(pos.x, rect.x_range(), self.x_range());
        let y = egui::remap_clamp(pos.y, rect.y_range(), self.y_range());

        Vec2::new(x, y)
    }
}

impl Widget for PointSelect<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click_and_drag());
        let painter = ui.painter();

        let visuals = ui.style().interact(&response);
        let line_stroke = visuals.fg_stroke;

        let circle_color = ui.style().visuals.widgets.active.fg_stroke.color;

        let line = |from: Pos2, to: Pos2| {
            painter.line_segment([from, to], line_stroke);
        };

        line(rect.center_top(), rect.center_bottom());
        line(rect.left_center(), rect.right_center());

        let circle_pos = self.value_to_ui_pos(&rect);
        painter.circle_filled(circle_pos, self.circle_radius, circle_color);

        if response.active {
            if let Some(mouse_pos) = ui.input().mouse.pos {
                *self.value = self.ui_pos_to_value(&rect, mouse_pos);
            }
        }

        response
    }
}
