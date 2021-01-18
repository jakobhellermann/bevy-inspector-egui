mod attributes;
mod inspectable;

#[proc_macro_derive(Inspectable, attributes(inspectable))]
pub fn inspectable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => inspectable::expand_struct(&input, data).into(),
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => unimplemented!(),
    }
}
