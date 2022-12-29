use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DataStruct, DataUnion, DeriveInput};

mod attributes;

/// Derive macro used to derive `InspectorOptions`
#[proc_macro_derive(InspectorOptions, attributes(inspector))]
pub fn inspectable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let result = match &input.data {
        syn::Data::Struct(data) => expand_struct(&input, data),
        syn::Data::Enum(data) => expand_enum(&input, data),
        syn::Data::Union(data) => expand_union(&input, data),
    };

    result.unwrap_or_else(|err| err.into_compile_error()).into()
}

fn expand_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let bevy_reflect = quote! { bevy_inspector_egui::__macro_exports::bevy_reflect };

    let fields = data
        .fields
        .iter()
        .filter(|field| !attributes::is_reflect_ignore_field(field))
        .enumerate()
        .filter_map(|(i, field)| {
            let ty = &field.ty;
            let attrs = match attributes::extract_inspector_attributes(&field.attrs) {
                Ok(attrs) => attrs,
                Err(e) => return Some(Err(e)),
            };
            if attrs.is_empty() {
                return None;
            }
            let attrs = attrs.into_iter().map(|attribute| {
                let name = attribute.lhs();
                let value = attribute.rhs();
                quote! {
                    field_options.#name = std::convert::Into::into(#value);
                }
            });

            Some(Ok(quote! {
                let mut field_options = <#ty as bevy_inspector_egui::inspector_options::InspectorOptionsType>::DeriveOptions::default();
                #(#attrs)*
                options.insert(bevy_inspector_egui::inspector_options::Target::Field(#i), <#ty as bevy_inspector_egui::inspector_options::InspectorOptionsType>::options_from_derive(field_options));
            }))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let type_name = &input.ident;

    Ok(quote! {
        impl #bevy_reflect::FromType<#type_name> for bevy_inspector_egui::InspectorOptions {
            fn from_type() -> Self {
                let mut options = bevy_inspector_egui::InspectorOptions::default();

                #(#fields)*

                options
            }
        }
    })
}

fn expand_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let bevy_reflect = quote! { bevy_inspector_egui::__macro_exports::bevy_reflect };

    let fields = data
        .variants
        .iter()
        .enumerate()
        .map(|(variant_index, variant)| {
            let attrs = variant
                .fields
                .iter()
                .filter(|field| !attributes::is_reflect_ignore_field(field))
                .enumerate()
                .filter_map(|(field_index, field)| {
                    let ty = &field.ty;
                    let attrs = match attributes::extract_inspector_attributes(&field.attrs) {
                        Ok(attrs) => attrs,
                        Err(e) => return Some(Err(e)),
                    };
                    if attrs.is_empty() {
                        return None;
                    }
                    let attrs = attrs.into_iter().map(|attribute| {
                        let name = attribute.lhs();
                        let value = attribute.rhs();
                        quote! {
                            field_options.#name = std::convert::Into::into(#value);
                        }
                    });

                    Some(Ok(quote! {
                        let mut field_options = <#ty as bevy_inspector_egui::inspector_options::InspectorOptionsType>::DeriveOptions::default();
                        #(#attrs)*
                        options.insert(
                            bevy_inspector_egui::inspector_options::Target::VariantField {
                                variant_index: #variant_index,
                                field_index: #field_index,
                            },
                            <#ty as bevy_inspector_egui::inspector_options::InspectorOptionsType>::options_from_derive(field_options)
                        );
                    }))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(attrs)
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let type_name = &input.ident;

    Ok(quote! {
        impl #bevy_reflect::FromType<#type_name> for bevy_inspector_egui::InspectorOptions {
            fn from_type() -> Self {
                let mut options = bevy_inspector_egui::InspectorOptions::default();

                #(#(#fields)*)*

                options
            }
        }
    })
}
fn expand_union(_: &DeriveInput, data: &DataUnion) -> syn::Result<TokenStream> {
    Err(syn::Error::new_spanned(
        data.union_token,
        "`InspectorOptions` for unions is not implemented",
    ))
}
