use bevy::{
    prelude::*,
    sprite::Anchor,
    render::mesh::{Indices, VertexAttributeValues},
    sprite::Mesh2dHandle,
};

use super::{OptionAttributes, Vec2dAttributes};
use crate::{Context, Inspectable};
use bevy_egui::egui::{self, Grid};

impl_for_struct_delegate_fields!(ColorMaterial: color, texture);

impl_for_struct_delegate_fields!(Sprite:
    color,
    flip_x,
    flip_y,
    custom_size with OptionAttributes { replacement: Some(|| Vec2::splat(50.0)), deletable: true, inner: Vec2dAttributes::positive() },
    anchor
);

impl_for_simple_enum!(Anchor:
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight:
    Custom Anchor::Custom(_) => Vec2::default()
);

impl_for_struct_delegate_fields!(bevy::sprite::Rect:
    min with Vec2dAttributes::integer(),
    max with Vec2dAttributes::integer(),
);
impl_for_struct_delegate_fields!(TextureAtlasSprite: color, index, flip_x, flip_y);

impl Inspectable for TextureAtlas {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        egui::Grid::new(context.id()).show(ui, |ui| {
            ui.label("texture");
            changed |= self
                .texture
                .ui(ui, Default::default(), &mut context.with_id(0));
            ui.end_row();

            ui.label("size");
            changed |= self.size.ui(ui, Vec2dAttributes::integer(), context);
            ui.end_row();

            ui.label("textures");
            ui.collapsing("Sections", |ui| {
                changed |= self
                    .textures
                    .ui(ui, Default::default(), &mut context.with_id(2));
            });
        });
        changed
    }
}

impl Inspectable for Mesh2dHandle {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        // Get the mesh from the handle
        if let Some(mesh) = context
            .world()
            .and_then(|world| world.get_resource::<Assets<Mesh>>())
            .and_then(|meshes| meshes.get(&self.0))
        {
            // Get 2D mesh attributes
            let indices = mesh.indices();
            let vertices = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
            let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR);

            #[allow(clippy::collapsible_match)]
            if let Some(((indices, vertices), colors)) = indices.zip(vertices).zip(colors) {
                // Convert the mesh into colored triangles
                if let Indices::U32(indices) = indices {
                    if let VertexAttributeValues::Float32x3(vertices) = vertices {
                        if let VertexAttributeValues::Float32x4(colors) = colors {
                            // Convert the mesh data into plot data
                            let vertices_and_colors = indices
                                .iter()
                                .map(|index| {
                                    let pos = vertices[*index as usize];
                                    let color = colors[*index as usize];
                                    (
                                        // Convert the bevy position to an egui position, discarding the Z value
                                        egui::plot::PlotPoint::new(pos[0], pos[1]),
                                        // Convert the bevy color to an egui color
                                        egui::Color32::from_rgba_unmultiplied(
                                            (color[0] * 255.0) as u8,
                                            (color[1] * 255.0) as u8,
                                            (color[2] * 255.0) as u8,
                                            (color[3] * 255.0) as u8,
                                        ),
                                    )
                                })
                                .collect::<Vec<_>>();

                            // Draw a grid with all the triangles
                            Grid::new(context.id()).show(ui, |ui| {
                                let plot = egui::plot::Plot::new("triangles")
                                    .legend(egui::plot::Legend::default())
                                    .data_aspect(0.8)
                                    .min_size(egui::Vec2::new(250.0, 250.0))
                                    .show_x(true)
                                    .show_y(true);

                                plot.show(ui, |plot_ui| {
                                    vertices_and_colors.chunks_exact(3).for_each(|triangle| {
                                        plot_ui.polygon(
                                            egui::plot::Polygon::new(
                                                egui::plot::PlotPoints::Owned(
                                                    triangle
                                                        .iter()
                                                        .map(|vertex_and_color| vertex_and_color.0)
                                                        .collect(),
                                                ),
                                            )
                                            // Add thicker strokes and reduce the fill
                                            // transparency
                                            .highlight(true)
                                            .color(triangle[0].1)
                                            // Use the color as the name so everything
                                            // with the same color will be grouped
                                            .name(
                                                format!(
                                                    "#{:02X?}{:02X?}{:02X?}{:02X?}",
                                                    triangle[0].1.r(),
                                                    triangle[0].1.g(),
                                                    triangle[0].1.b(),
                                                    triangle[0].1.a()
                                                ),
                                            ),
                                        );
                                    });
                                });
                            });
                        }
                    }
                }
            }
        }

        false
    }
}
