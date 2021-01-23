mod attributes;
mod inspectable_enum;
mod inspectable_struct;

#[proc_macro_derive(Inspectable, attributes(inspectable))]
pub fn inspectable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => inspectable_struct::expand_struct(&input, data, false).into(),
        syn::Data::Enum(data) => inspectable_enum::expand_enum(&input, data).into(),
        syn::Data::Union(_) => unimplemented!(),
    }
}

#[proc_macro_derive(InspectableWithContext, attributes(inspectable))]
pub fn inspectable_with_context(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => inspectable_struct::expand_struct(&input, data, true).into(),
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    }
}
