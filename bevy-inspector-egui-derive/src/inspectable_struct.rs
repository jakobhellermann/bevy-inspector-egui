use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::utils;

pub fn expand_struct(
    derive_input: &syn::DeriveInput,
    data: &syn::DataStruct,
) -> syn::Result<TokenStream> {
    let name = &derive_input.ident;

    let fields: Vec<_> = data
        .fields
        .iter()
        .map(|field| {
            let attributes = crate::attributes::inspectable_attributes(&field.attrs)?;

            if attributes.default.is_some() {
                let span = match field.attrs.as_slice() {
                    [attr] => Some(attr.span()),
                    [start, .., end] => dbg!(start.span().join(end.span())),
                    _ => None,
                };
                let span = span
                    .or_else(|| field.ident.as_ref().map(|i| i.span()))
                    .unwrap_or(field.ty.span());

                return Err(syn::Error::new(
                    span,
                    "#[inspectable(default = <expr>)] is only for enums",
                ));
            }

            Ok((field, attributes))
        })
        .collect::<syn::Result<_>>()?;

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

    let generic_for_impl = utils::with_inspectable_bound(&derive_input.generics);
    let (impl_generics, ty_generics, where_clause) = generic_for_impl.split_for_impl();

    Ok(quote! {
        #[allow(clippy::all)]
        impl #impl_generics bevy_inspector_egui::Inspectable for #name #ty_generics #where_clause {
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
    })
}
