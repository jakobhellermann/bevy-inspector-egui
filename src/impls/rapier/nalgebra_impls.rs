use bevy_egui::egui;
use nalgebra::{
    storage::StorageMut, Complex, Dim, Isometry, Matrix, Quaternion, SVector, Scalar, Translation,
    Unit,
};

use crate::Inspectable;

impl<T: Scalar + Inspectable, R: Dim, C: Dim, S: StorageMut<T, R, C>> Inspectable
    for Matrix<T, R, C, S>
{
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _: Self::Attributes,
        context: &crate::Context,
    ) -> bool {
        let mut changed = false;

        match (R::try_to_usize(), C::try_to_usize()) {
            (Some(1), Some(1)) => {
                let value = self.get_mut((0, 0)).unwrap();
                changed |= value.ui(ui, Default::default(), context);
            }
            (Some(1), Some(len)) | (Some(len), Some(1)) => {
                ui.scope(|ui| {
                    ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);
                    ui.columns(len, |ui| {
                        for (i, value) in self.iter_mut().enumerate() {
                            changed |= value.ui(&mut ui[i], Default::default(), context);
                        }
                    });
                });
            }
            (_, _) => {
                ui.vertical(|ui| {
                    for mut row in self.row_iter_mut() {
                        changed |= row.ui(ui, Default::default(), context);
                    }
                });
            }
        }

        changed
    }
}

impl<T: Scalar + Inspectable, const D: usize> Inspectable for Translation<T, D> {
    type Attributes = <SVector<T, D> as Inspectable>::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) -> bool {
        self.vector.ui(ui, options, context)
    }
}

impl Inspectable for Unit<Quaternion<f32>> {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _: Self::Attributes,
        context: &crate::Context,
    ) -> bool {
        let vec: bevy::math::Vec4 = (*self.as_vector()).into();
        let mut quat = bevy::math::Quat::from(vec);
        let changed = quat.ui(ui, Default::default(), context);
        if changed {
            let vec: bevy::math::Vec4 = quat.into();
            let quat = Quaternion::<f32>::from_vector(vec.into());
            let quat = Unit::<Quaternion<f32>>::new_normalize(quat);
            *self = quat;
        }

        changed
    }
}

impl Inspectable for Unit<Complex<f32>> {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: Self::Attributes,
        _: &crate::Context,
    ) -> bool {
        let mut angle = self.angle();
        let changed = ui.drag_angle_tau(&mut angle).changed();
        if changed {
            *self = Unit::<Complex<f32>>::from_angle(angle);
        }
        changed
    }
}

impl<T: Inspectable + Scalar, R: Inspectable, const D: usize> Inspectable for Isometry<T, R, D> {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _: Self::Attributes,
        context: &crate::Context,
    ) -> bool {
        let mut changed = false;

        ui.vertical_centered(|ui| {
            crate::egui::Grid::new(context.id()).show(ui, |ui| {
                ui.label("Rotation");
                changed |= self
                    .rotation
                    .ui(ui, Default::default(), &context.with_id(0));
                ui.end_row();
                ui.label("Translation");
                changed |= self
                    .translation
                    .ui(ui, Default::default(), &context.with_id(1));
                ui.end_row();
            });
        });

        changed
    }
}
