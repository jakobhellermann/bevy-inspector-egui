pub fn field_accessor(field: &syn::Field, i: usize) -> syn::Member {
    match &field.ident {
        Some(name) => syn::Member::Named(name.clone()),
        None => syn::Member::Unnamed(syn::Index::from(i)),
    }
}
