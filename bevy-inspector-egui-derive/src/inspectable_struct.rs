use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn expand_struct(derive_input: &syn::DeriveInput, data: &syn::DataStruct) -> TokenStream {
    let name = &derive_input.ident;
    let id = name;

    let fields = data.fields.iter().enumerate().map(|(i, field)| {
        let ty = &field.ty;

        let field_label = field_label(field, i);
        let accessor = field_accessor(field, i);

        let attributes = crate::attributes::inspectable_attributes(&field.attrs);
        let custom_options = attributes.fold(
            quote! {let mut custom_options = <#ty as bevy_inspector_egui::InspectableWidget>::FieldOptions::default();},
            |acc, (name, expr)| {
                quote! {
                    #acc
                    custom_options.#name = #expr;
                }
            },
        );

        quote! {
            ui.label(#field_label);
            #custom_options
            let options = bevy_inspector_egui::Options::new(custom_options);
            <#ty as bevy_inspector_egui::InspectableWidget>::ui(&mut self.#accessor, ui, options);
            ui.end_row();
        }
    });

    quote! {
        impl bevy_inspector_egui::InspectableWidget for #name {
            type FieldOptions = ();


            fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: bevy_inspector_egui::Options<Self::FieldOptions>) {
                use bevy_inspector_egui::egui;

                let grid = egui::Grid::new(stringify!(#id));
                grid.show(ui, |ui| {
                    #(#fields)*
                });
            }
        }
    }
}

fn field_accessor(field: &syn::Field, i: usize) -> TokenStream {
    match &field.ident {
        Some(name) => name.to_token_stream(),
        None => syn::Index::from(i).to_token_stream(),
    }
}

fn field_label(field: &syn::Field, i: usize) -> String {
    match &field.ident {
        Some(name) => name.to_string(),
        None => i.to_string(),
    }
}
