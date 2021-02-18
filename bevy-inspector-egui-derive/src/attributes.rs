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
        } else {
            Ok(InspectableAttribute::Tag(ident))
        }
    };

    input
        .parse_terminated::<_, syn::Token![,]>(parse_attribute)
        .map(IntoIterator::into_iter)
}

/// extracts [(min, 8), (field, vec2(1.0, 1.0))] from `#[inspectable(min = 8, field = vec2(1.0, 1.0))]`,
fn extract_inspectable_attributes(
    attrs: &[syn::Attribute],
) -> impl Iterator<Item = InspectableAttribute> + '_ {
    attrs
        .iter()
        .filter(|attr| attr.path.get_ident().map_or(false, |p| p == "inspectable"))
        .flat_map(|attr| attr.parse_args_with(parse_inspectable_attributes).unwrap())
}

#[derive(Default)]
pub struct InspectableAttributes {
    pub collapse: bool,
    pub label: Option<String>,
    pub custom_attributes: Vec<InspectableAttribute>,
}

pub fn inspectable_attributes(attrs: &[syn::Attribute]) -> InspectableAttributes {
    let mut all = InspectableAttributes::default();

    let (builtin_attributes, custom_attributes): (Vec<_>, Vec<_>) =
        extract_inspectable_attributes(attrs).partition(InspectableAttribute::is_builtin);

    // builtins
    for builtin_attribute in builtin_attributes {
        match builtin_attribute {
            InspectableAttribute::Tag(syn::Member::Named(ident)) if ident == "collapse" => {
                all.collapse = true;
            }
            #[rustfmt::skip]
            InspectableAttribute::Assignment(syn::Member::Named(ident), expr) if ident == "label" => {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(str), .. }) = expr {
                    all.label = Some(str.value());
                } else {
                    panic!("label needs to be a string literal");
                };
            }
            InspectableAttribute::Tag(name) | InspectableAttribute::Assignment(name, _) => {
                match name {
                    syn::Member::Named(name) => panic!("unknown attributes '{}'", name),
                    syn::Member::Unnamed(_) => unreachable!(),
                }
            }
        }
    }

    all.custom_attributes = custom_attributes;
    all
}
