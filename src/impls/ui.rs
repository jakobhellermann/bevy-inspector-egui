use bevy::prelude::*;
use bevy_egui::egui;

use crate::Inspectable;

use super::NumberAttributes;

impl_for_simple_enum!(Display: Flex, None);
impl_for_simple_enum!(bevy::ui::FocusPolicy: Block, Pass);
impl_for_simple_enum!(VerticalAlign: Top, Center, Bottom);
impl_for_simple_enum!(HorizontalAlign: Left, Center, Right);
impl_for_simple_enum!(PositionType: Absolute, Relative);
impl_for_simple_enum!(Direction: Inherit, LeftToRight, RightToLeft);
impl_for_simple_enum!(FlexDirection: Row, Column, RowReverse, ColumnReverse);
impl_for_simple_enum!(FlexWrap: NoWrap, Wrap, WrapReverse);
impl_for_simple_enum!(AlignItems: FlexStart, FlexEnd, Center, Baseline, Stretch);
impl_for_simple_enum!(
    AlignSelf: Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch
);
impl_for_simple_enum!(
    AlignContent: FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround
);
impl_for_simple_enum!(
    JustifyContent: FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly
);

impl_for_struct_delegate_fields!(TextAlignment: vertical, horizontal);
impl_for_struct_delegate_fields!(TextStyle: font, font_size, color);
impl_for_struct_delegate_fields!(TextSection: value, style);
impl_for_struct_delegate_fields!(Text: sections, alignment);

impl_for_struct_delegate_fields!(
    Style:
    //display,
    position_type,
    direction,
    flex_direction,
    flex_wrap,
    align_items,
    align_self,
    align_content,
    justify_content,
    //position,
    //margin,
    //padding,
    //border,
    flex_grow with NumberAttributes::positive(),
    flex_shrink with NumberAttributes::positive(),
    flex_basis,
    size,
    min_size,
    max_size,
    // aspect_ratio,
);

impl<T: Inspectable + Reflect + PartialEq> Inspectable for Size<T> {
    type Attributes = T::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &mut crate::Context,
    ) -> bool {
        let mut changed = false;
        ui.vertical_centered(|ui| {
            crate::egui::Grid::new(context.id()).show(ui, |ui| {
                ui.label("width");
                changed |= self.width.ui(ui, options.clone(), &mut context.with_id(0));
                ui.end_row();

                ui.label("height");
                changed |= self.height.ui(ui, options, &mut context.with_id(1));
                ui.end_row();
            });
            ui.separator();
        });
        changed
    }
}

impl Inspectable for Val {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _: Self::Attributes,
        context: &mut crate::Context,
    ) -> bool {
        use std::mem::discriminant;

        let selected = match self {
            Val::Undefined => "Undefined",
            Val::Auto => "Auto",
            Val::Px(_) => "Px",
            Val::Percent(_) => "Percent",
        };

        enum WhatToShow {
            Px,
            Percent,
        }

        let mut what_to_show = match self {
            Val::Px(_) => Some(WhatToShow::Px),
            Val::Percent(_) => Some(WhatToShow::Percent),
            _ => None,
        };

        let mut changed = false;

        ui.columns(2, |ui| {
            egui::ComboBox::from_id_source(context.id())
                .selected_text(selected)
                .show_ui(&mut ui[0], |ui| {
                    if ui
                        .selectable_label(*self == Val::Undefined, "Undefined")
                        .clicked()
                    {
                        *self = Val::Undefined;
                        what_to_show = None;
                        changed = true;
                    }
                    if ui.selectable_label(*self == Val::Auto, "Auto").clicked() {
                        *self = Val::Auto;
                        what_to_show = None;
                        changed = true;
                    }
                    let is_px = discriminant(self) == discriminant(&Val::Px(0.0));
                    if ui.selectable_label(is_px, "Px").clicked() {
                        what_to_show = Some(WhatToShow::Px);
                        changed = true;
                    }
                    let is_pct = discriminant(self) == discriminant(&Val::Percent(0.0));
                    if ui.selectable_label(is_pct, "Percent").clicked() {
                        what_to_show = Some(WhatToShow::Percent);
                        changed = true;
                    }
                });

            match what_to_show {
                Some(WhatToShow::Px) => {
                    let mut value = match self {
                        Val::Px(val) => *val,
                        _ => 0.0,
                    };

                    let attrs = NumberAttributes {
                        min: Some(0.0),
                        speed: 10.0,
                        ..Default::default()
                    };
                    changed |= value.ui(&mut ui[1], attrs, context);
                    *self = Val::Px(value);
                }
                Some(WhatToShow::Percent) => {
                    let mut value = match self {
                        Val::Percent(val) => *val,
                        _ => 0.0,
                    };

                    let attrs = NumberAttributes {
                        min: Some(0.0),
                        max: Some(100.0),
                        ..Default::default()
                    };
                    changed |= value.ui(&mut ui[1], attrs, context);
                    *self = Val::Percent(value);
                }
                None => {}
            }
        });
        changed
    }
}
