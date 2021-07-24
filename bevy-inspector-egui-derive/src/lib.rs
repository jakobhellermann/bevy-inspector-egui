mod attributes;
mod inspectable_enum;
mod inspectable_struct;
mod utils;

#[proc_macro_derive(Inspectable, attributes(inspectable))]
pub fn inspectable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let result = match &input.data {
        syn::Data::Struct(data) => inspectable_struct::expand_struct(&input, data),
        syn::Data::Enum(data) => inspectable_enum::expand_enum(&input, data),
        syn::Data::Union(_) => unimplemented!(),
    };

    result.unwrap_or_else(|err| err.into_compile_error()).into()
}
