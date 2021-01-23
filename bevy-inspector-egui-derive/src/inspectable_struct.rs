use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::attributes::InspectableAttribute;

pub fn expand_struct(
    derive_input: &syn::DeriveInput,
    data: &syn::DataStruct,
    with_context: bool,
) -> TokenStream {
    let name = &derive_input.ident;
    let id = name;

    let inspectable_trait = match with_context {
        false => quote! { bevy_inspector_egui::Inspectable },
        true => quote! { bevy_inspector_egui::InspectableWithContext },
    };
    let inspectable_ui_fn = match with_context {
        false => quote! { ui },
        true => quote! { ui_with_context },
    };
    let (context_param_decl, context_param) = match with_context {
        false => (quote! {}, quote! {}),
        true => (
            quote! { context: &bevy_inspector_egui::Context },
            quote! { context},
        ),
    };

    let fields = data.fields.iter().enumerate().map(|(i, field)| {
        let ty = &field.ty;

        let field_label = field_label(field, i);
        let accessor = field_accessor(field, i);

        let (builtin_attributes, custom_attributes): (Vec<_>, Vec<_>) = crate::attributes::inspectable_attributes(&field.attrs).partition(InspectableAttribute::is_builtin);

        // builtins
        let mut collapse = false;
        let mut custom_label = None;
        for builtin_attribute in builtin_attributes {
            match builtin_attribute {
                InspectableAttribute::Tag(ident) if ident == "collapse" => collapse = true,
                InspectableAttribute::Assignment(ident, expr) if ident == "label" => custom_label = Some(expr),
                InspectableAttribute::Tag(name) | InspectableAttribute::Assignment(name, _) => panic!("unknown attributes '{}'", name),
            }
        }
        let field_label  = match custom_label {
            Some(label) => label.to_token_stream(),
            None => quote! { #field_label },
        };

        // user specified options
        let options = custom_attributes.iter().fold(
            quote! { let mut options = <#ty as #inspectable_trait>::Attributes::default(); },
            |acc,attribute| {
                let value = attribute.as_expr();
                let name = attribute.ident();

                quote! {
                    #acc
                    options.#name = std::convert::From::from(#value);
                }
            },
        );

        let ui = quote! {
            #options
            <#ty as #inspectable_trait>::#inspectable_ui_fn(&mut self.#accessor, ui, options, #context_param);
        };

        let ui = match collapse {
            true => quote! { ui.collapsing(#field_label, |ui| {#ui}); },
            false => ui,
        };

        quote! {
            ui.label(#field_label);
            #ui
            ui.end_row();
        }

    });

    quote! {
        impl #inspectable_trait for #name {
            type Attributes = ();


            fn #inspectable_ui_fn(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: Self::Attributes, #context_param_decl) {
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
