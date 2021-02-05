use proc_macro2::TokenStream;
use quote::quote;

pub enum InspectableAttribute {
    Assignment(syn::Member, syn::Expr),
    Tag(syn::Member),
}
impl InspectableAttribute {
    pub fn lhs(&self) -> &syn::Member {
        match self {
            InspectableAttribute::Assignment(member, _) => member,
            InspectableAttribute::Tag(member) => member,
        }
    }

    pub fn rhs(&self) -> TokenStream {
        match self {
            InspectableAttribute::Assignment(_, expr) => quote! { #expr },
            InspectableAttribute::Tag(_) => quote! { true },
        }
    }

    pub fn is_builtin(&self) -> bool {
        let ident = match self.lhs() {
            syn::Member::Named(ident) => ident,
            syn::Member::Unnamed(_) => return false,
        };

        ident == "label" || ident == "collapse"
    }
}

fn parse_inspectable_attributes(
    input: syn::parse::ParseStream,
) -> syn::Result<impl Iterator<Item = InspectableAttribute>> {
    let parse_attribute = |input: syn::parse::ParseStream| {
        let ident: syn::Member = input.parse()?;
        if input.peek(syn::Token![=]) {
            let _eq_token: syn::Token![=] = input.parse()?;
            let expr: syn::Expr = input.parse()?;
            Ok(InspectableAttribute::Assignment(ident, expr))
        } else if input.is_empty() {
            Ok(InspectableAttribute::Tag(ident))
        } else {
            panic!("could not parse attribute {:?}", ident);
        }
    };

    input
        .parse_terminated::<_, syn::Token![,]>(parse_attribute)
        .map(IntoIterator::into_iter)
}

/// extracts [(min, 8), (field, vec2(1.0, 1.0))] from `#[inspectable(min = 8, field = vec2(1.0, 1.0))]`,
pub fn inspectable_attributes(
    attrs: &[syn::Attribute],
) -> impl Iterator<Item = InspectableAttribute> + '_ {
    attrs
        .iter()
        .filter(|attr| attr.path.get_ident().map_or(false, |p| p == "inspectable"))
        .flat_map(|attr| attr.parse_args_with(parse_inspectable_attributes).unwrap())
}
