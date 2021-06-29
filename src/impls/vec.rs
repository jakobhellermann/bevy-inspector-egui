use std::ops::RangeInclusive;

use crate::{Context, Inspectable};
use bevy::math::{Vec2, Vec3, Vec4};
use bevy_egui::egui::{self, containers, Rect};
use egui::{Pos2, Sense, Widget};

use super::NumberAttributes;

#[derive(Debug, Default, Clone)]
pub struct Vec2dAttributes {
    pub visual: bool,
    pub min: Option<Vec2>,
    pub max: Option<Vec2>,
    pub speed: f32,
}
impl Vec2dAttributes {
    pub(crate) fn integer() -> Self {
        Vec2dAttributes {
            speed: 1.0,
            ..Default::default()
        }
    }
}

impl Inspectable for Vec2 {
    type Attributes = Vec2dAttributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) -> bool {
        if options.visual {
            point_select(self, ui, options)
        } else {
            let mut changed = false;
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);

                ui.columns(2, |ui| {
                    let x_attrs = NumberAttributes {
                        min: options.min.map(|vec| vec.x),
                        max: options.max.map(|vec| vec.x),
                        speed: options.speed,
                        ..Default::default()
                    };
                    let y_attrs = NumberAttributes {
                        min: options.min.map(|vec| vec.y),
                        max: options.max.map(|vec| vec.y),
                        speed: options.speed,
                        ..Default::default()
                    };
                    changed |= self.x.ui(&mut ui[0], x_attrs, context);
                    changed |= self.y.ui(&mut ui[1], y_attrs, context);
                });
            });
            changed
        }
    }
}

fn point_select(
    value: &mut Vec2,
    ui: &mut egui::Ui,
    options: Vec2dAttributes,
) -> bool {
    let range = match (options.min, options.max) {
        (Some(min), Some(max)) => min..=max,
        (Some(min), None) => min..=Vec2::splat(0.0),
        (None, Some(max)) => Vec2::splat(0.0)..=max,
        (None, None) => Vec2::splat(-100.0)..=Vec2::splat(100.0),
    };

    let mut frame = containers::Frame::dark_canvas(&ui.style());
    frame.margin = egui::Vec2::ZERO;

    frame
        .show(ui, |ui| {
            let widget = PointSelect::new(value, range, 80.0);
            ui.add(widget).changed()
        })
        .inner
}

struct PointSelect<'a> {
    size: egui::Vec2,
    circle_radius: f32,
    range: RangeInclusive<Vec2>,
    value: &'a mut Vec2,
}
impl<'a> PointSelect<'a> {
    fn new(
        value: &'a mut Vec2,
        range: RangeInclusive<Vec2>,
        size: f32,
    ) -> Self {
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

    fn value_to_ui_pos(
        &self,
        rect: &Rect,
    ) -> Pos2 {
        let x = egui::remap_clamp(self.value.x, self.x_range(), rect.x_range());
        let y = egui::remap_clamp(self.value.y, self.y_range(), rect.y_range());
        Pos2::new(x, y)
    }
    fn ui_pos_to_value(
        &self,
        rect: &Rect,
        pos: Pos2,
    ) -> Vec2 {
        let x = egui::remap_clamp(pos.x, rect.x_range(), self.x_range());
        let y = egui::remap_clamp(pos.y, rect.y_range(), self.y_range());

        Vec2::new(x, y)
    }
}

impl Widget for PointSelect<'_> {
    fn ui(
        self,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        let (rect, mut response) = ui.allocate_exact_size(self.size, Sense::click_and_drag());
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

        if response.dragged() {
            if let Some(mouse_pos) = ui.input().pointer.interact_pos() {
                *self.value = self.ui_pos_to_value(&rect, mouse_pos);
            }
            response.mark_changed();
        }

        response
    }
}

impl Inspectable for Vec3 {
    type Attributes = NumberAttributes<Vec3>;

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) -> bool {
        let mut changed = false;
        ui.scope(|ui| {
            ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);

            ui.columns(3, |ui| {
                changed |= self.x.ui(&mut ui[0], options.map(|vec| vec.x), context);
                changed |= self.y.ui(&mut ui[1], options.map(|vec| vec.y), context);
                changed |= self.z.ui(&mut ui[2], options.map(|vec| vec.z), context);
            });
        });
        changed
    }
}

impl Inspectable for Vec4 {
    type Attributes = NumberAttributes<Vec4>;

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) -> bool {
        let mut changed = false;
        ui.scope(|ui| {
            ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);

            ui.columns(4, |ui| {
                changed |= self.x.ui(&mut ui[0], options.map(|vec| vec.x), context);
                changed |= self.y.ui(&mut ui[1], options.map(|vec| vec.y), context);
                changed |= self.z.ui(&mut ui[2], options.map(|vec| vec.z), context);
                changed |= self.w.ui(&mut ui[3], options.map(|vec| vec.w), context);
            });
        });
        changed
    }
}
