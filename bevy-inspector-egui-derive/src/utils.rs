pub fn field_accessor(field: &syn::Field, i: usize) -> syn::Member {
    match &field.ident {
        Some(name) => syn::Member::Named(name.clone()),
        None => syn::Member::Unnamed(syn::Index::from(i)),
    }
}

pub fn field_label(field: &syn::Field, i: usize) -> String {
    match &field.ident {
        Some(name) => name.to_string(),
        None => i.to_string(),
    }
}

pub fn with_inspectable_bound(
    generics: &syn::Generics,
    override_where_clause: &Option<syn::WhereClause>,
) -> syn::Generics {
    let mut generics = generics.clone();

    if override_where_clause.is_none() {
        for param in generics.type_params_mut() {
            param
                .bounds
                .push(syn::parse_quote!(bevy_inspector_egui::Inspectable));
        }
    }

    generics
}
