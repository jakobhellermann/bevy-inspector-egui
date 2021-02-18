use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn expand_struct(derive_input: &syn::DeriveInput, data: &syn::DataStruct) -> TokenStream {
    let name = &derive_input.ident;

    let field_setup = data.fields.iter().map(|field| {
        let ty = &field.ty;

        quote! {
            <#ty as bevy_inspector_egui::Inspectable>::setup(app);
        }
    });

    let fields = data.fields.iter().enumerate().map(|(i, field)| {
        let ty = &field.ty;

        let field_label = field_label(field, i);
        let accessor = field_accessor(field, i);

        let attributes = crate::attributes::inspectable_attributes(&field.attrs);

        let field_label  = match &attributes.label {
            Some(label) => label.to_token_stream(),
            None => quote! { #field_label },
        };

        // user specified options
        let options = attributes.custom_attributes.iter().fold(
            quote! { let mut options = <#ty as bevy_inspector_egui::Inspectable>::Attributes::default(); },
            |acc,attribute| {
                let value = attribute.rhs();
                let name = attribute.lhs();

                quote! {
                    #acc
                    options.#name = std::convert::From::from(#value);
                }
            },
        );

        let ui = quote! {
            #options
            <#ty as bevy_inspector_egui::Inspectable>::ui(&mut self.#accessor, ui, options, &context.with_id(#i as u64));
        };

        let ui = if attributes.collapse {
            quote! { bevy_inspector_egui::egui::CollapsingHeader::new(#field_label).id_source(#i as u64).show(ui, |ui| { #ui }); }
        } else {
            ui
        };

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


            fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: Self::Attributes, context: &bevy_inspector_egui::Context) {
                use bevy_inspector_egui::egui;

                ui.vertical_centered(|ui| {
                    let grid = egui::Grid::new(context.id());
                    grid.show(ui, |ui| {
                        #(#fields)*
                    });
                });
            }

            fn setup(app: &mut bevy::prelude::AppBuilder) {
                #(#field_setup)*
            }
        }
    }
}

fn field_accessor(field: &syn::Field, i: usize) -> syn::Member {
    match &field.ident {
        Some(name) => syn::Member::Named(name.clone()),
        None => syn::Member::Unnamed(syn::Index::from(i)),
    }
}

fn field_label(field: &syn::Field, i: usize) -> String {
    match &field.ident {
        Some(name) => name.to_string(),
        None => i.to_string(),
    }
}
