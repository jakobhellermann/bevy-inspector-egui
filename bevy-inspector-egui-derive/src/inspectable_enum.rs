use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_enum(derive_input: &syn::DeriveInput, data: &syn::DataEnum) -> TokenStream {
    let name = &derive_input.ident;
    let id = name;

    let variants = data.variants.iter().map(|variant| match variant.fields {
        syn::Fields::Named(_) => todo!("named fields"),
        syn::Fields::Unnamed(_) => todo!("unnamed fields"),
        syn::Fields::Unit => {
            let ident = &variant.ident;
            quote! {
                ui.selectable_value(self, #name::#ident, format!("{:?}", #name::#ident));
            }
        }
    });

    quote! {
        impl bevy_inspector_egui::InspectableWidget for #name {
            type FieldOptions = ();


            fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: Options<Self::FieldOptions>) {
                use bevy_inspector_egui::egui;

                let id = ui.make_persistent_id(stringify!(#id));
                egui::combo_box(ui, id, format!("{:?}", self), |ui| {
                    #(#variants)*
                });
            }
        }
    }
}
