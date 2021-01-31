use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_enum(derive_input: &syn::DeriveInput, data: &syn::DataEnum) -> TokenStream {
    let name = &derive_input.ident;

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
        impl bevy_inspector_egui::Inspectable for #name {
            type Attributes = ();


            fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: Self::Attributes, context: &bevy_inspector_egui::Context) {
                use bevy_inspector_egui::egui;

                egui::combo_box(ui, context.id(), format!("{:?}", self), |ui| {
                    #(#variants)*
                });
            }
        }
    }
}
