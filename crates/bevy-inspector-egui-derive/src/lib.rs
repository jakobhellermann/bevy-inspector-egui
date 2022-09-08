use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DataStruct, DataUnion, DeriveInput};

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
        .enumerate()
        .filter_map(|(i, field)| {
            let ty = &field.ty;
            let attrs = match collect_attrs(field) {
                Ok(attrs) => attrs,
                Err(e) => return Some(Err(e)),
            };
            if attrs.is_empty() {
                return None;
            }
            let attrs = attrs.into_iter().map(|(name, value)| quote! {
                field_options.#name = std::convert::Into::into(#value);
            });

            Some(Ok(quote! {
                let mut field_options = <#ty as bevy_inspector_egui::options::InspectorOptionsType>::TypedOptions::default();
                #(#attrs)*
                options.insert(bevy_inspector_egui::options::Target::Field(#i), <#ty as bevy_inspector_egui::options::InspectorOptionsType>::Options::from(field_options));
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
        .map(|variant| {
            let variant_name = variant.ident.to_string();
            let attrs = variant
                .fields
                .iter()
                .enumerate()
                .filter_map(|(i, field)| {
                    let ty = &field.ty;
                    let attrs = match collect_attrs(field) {
                        Ok(attrs) => attrs,
                        Err(e) => return Some(Err(e)),
                    };
                    if attrs.is_empty() {
                        return None;
                    }
                    let attrs = attrs.into_iter().map(|(name, value)| quote! {
                        field_options.#name = std::convert::Into::into(#value);
                    });

                    Some(Ok(quote! {
                        let mut field_options = <#ty as bevy_inspector_egui::options::InspectorOptionsType>::TypedOptions::default();
                        #(#attrs)*
                        options.insert(bevy_inspector_egui::options::Target::VariantField(std::borrow::Cow::Borrowed(#variant_name), #i), <#ty as bevy_inspector_egui::options::InspectorOptionsType>::Options::from(field_options));
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

fn collect_attrs(field: &syn::Field) -> Result<Vec<(syn::Ident, syn::Lit)>, syn::Error> {
    let fields = field
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("inspector"))
        .map(|attr| {
            let meta = attr.parse_meta()?;
            let list = meta_list(meta)?;
            let key_value = list
                .nested
                .into_iter()
                .map(|meta| {
                    let name_value = nested_meta_meta(meta).and_then(meta_name_value)?;
                    let path = name_value
                        .path
                        .get_ident()
                        .ok_or_else(|| {
                            syn::Error::new_spanned(
                                &name_value.path,
                                "expected a simple identifier",
                            )
                        })?
                        .clone();
                    Ok((path, name_value.lit))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(key_value)
        })
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .flatten();
    Ok(fields.collect())
}
fn meta_list(meta: syn::Meta) -> Result<syn::MetaList, syn::Error> {
    match meta {
        syn::Meta::Path(path) => Err(syn::Error::new_spanned(path, "unexpected path")),
        syn::Meta::NameValue(name_value) => Err(syn::Error::new_spanned(
            name_value,
            "unexpected name value pair",
        )),
        syn::Meta::List(list) => Ok(list),
    }
}

fn meta_name_value(meta: syn::Meta) -> Result<syn::MetaNameValue, syn::Error> {
    match meta {
        syn::Meta::Path(path) => Err(syn::Error::new_spanned(path, "unexpected path")),
        syn::Meta::List(list) => Err(syn::Error::new_spanned(list, "unexpected list")),
        syn::Meta::NameValue(name_value) => Ok(name_value),
    }
}

fn nested_meta_meta(meta: syn::NestedMeta) -> Result<syn::Meta, syn::Error> {
    match meta {
        syn::NestedMeta::Lit(lit) => Err(syn::Error::new_spanned(lit, "unexpected literal")),
        syn::NestedMeta::Meta(meta) => Ok(meta),
    }
}
