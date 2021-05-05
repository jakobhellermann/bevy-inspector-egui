use proc_macro2::TokenStream;
use quote::quote;

use crate::utils;

pub fn expand_struct(derive_input: &syn::DeriveInput, data: &syn::DataStruct) -> TokenStream {
    let name = &derive_input.ident;

    let fields: Vec<_> = data
        .fields
        .iter()
        .map(|field| {
            let attributes = crate::attributes::inspectable_attributes(&field.attrs);
            (field, attributes)
        })
        .collect();

    let field_setup = fields.iter().map(|(field, attributes)| {
        if attributes.ignore {
            return quote! {};
        }

        let ty = &field.ty;

        quote! {
            <#ty as bevy_inspector_egui::Inspectable>::setup(app);
        }
    });

    let fields = fields.iter().enumerate().map(|(i, (field, attributes))| {
        if attributes.ignore {
            return quote! {};
        }

        let ty = &field.ty;

        let accessor = utils::field_accessor(field, i);
        let field_label = utils::field_label(field, i);
        let field_label = attributes.label(&field_label);

        if attributes.default.is_some() {
            panic!("#[inspectable(default = <expr>)] is only for enums");
        }

        // user specified options
        let options = attributes.create_options_struct(ty);

        let ui = quote! {
            let options = #options;
            changed |= <#ty as bevy_inspector_egui::Inspectable>::ui(&mut self.#accessor, ui, options, &context.with_id(#i as u64));
        };
        let ui = attributes.decorate_ui(ui, field_label, i);

        quote! {
            ui.label(#field_label);
            #ui
            ui.end_row();
        }
    });

    quote! {
        #[allow(clippy::all)]
        impl bevy_inspector_egui::Inspectable for #name {
            type Attributes = ();


            fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: Self::Attributes, context: &bevy_inspector_egui::Context) -> bool {
                use bevy_inspector_egui::egui;

                let mut changed = false;
                ui.vertical_centered(|ui| {
                    let grid = egui::Grid::new(context.id());
                    grid.show(ui, |ui| {
                        #(#fields)*
                    });
                });
                changed
            }

            fn setup(app: &mut bevy::prelude::AppBuilder) {
                #(#field_setup)*
            }
        }
    }
}
