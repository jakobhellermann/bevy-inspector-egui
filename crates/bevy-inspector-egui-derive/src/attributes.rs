use proc_macro2::TokenStream;
use quote::quote;

fn is_reflect_ignore(attribute: &syn::Attribute) -> bool {
    if !attribute.path().is_ident("reflect") {
        return false;
    }

    let mut ignore = false;
    let _ = attribute.parse_nested_meta(|meta| {
        ignore = meta.path.is_ident("ignore");
        Ok(())
    });
    ignore
}
pub fn is_reflect_ignore_field(field: &syn::Field) -> bool {
    field.attrs.iter().any(is_reflect_ignore)
}

pub enum InspectorAttribute {
    Assignment(syn::Member, syn::Expr),
    Tag(syn::Member),
}

impl InspectorAttribute {
    pub fn lhs(&self) -> &syn::Member {
        match self {
            InspectorAttribute::Assignment(member, _) => member,
            InspectorAttribute::Tag(member) => member,
        }
    }

    pub fn rhs(&self) -> TokenStream {
        match self {
            InspectorAttribute::Assignment(_, expr) => quote! { #expr },
            InspectorAttribute::Tag(_) => quote! { true },
        }
    }
}

fn parse_inspectable_attributes(
    input: syn::parse::ParseStream,
) -> syn::Result<impl Iterator<Item = InspectorAttribute>> {
    let parse_attribute = |input: syn::parse::ParseStream| {
        let ident: syn::Member = input.parse()?;
        if input.peek(syn::Token![=]) {
            let _eq_token: syn::Token![=] = input.parse()?;
            let expr: syn::Expr = input.parse()?;
            Ok(InspectorAttribute::Assignment(ident, expr))
        } else {
            Ok(InspectorAttribute::Tag(ident))
        }
    };

    input
        .parse_terminated(parse_attribute, syn::Token![,])
        .map(IntoIterator::into_iter)
}

pub fn extract_inspector_attributes(
    attrs: &[syn::Attribute],
) -> syn::Result<Vec<InspectorAttribute>> {
    Ok(attrs
        .iter()
        .filter(|attr| attr.path().get_ident().is_some_and(|p| p == "inspector"))
        .map(|attr| attr.parse_args_with(parse_inspectable_attributes))
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect())
}
