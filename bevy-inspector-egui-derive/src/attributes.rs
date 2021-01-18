pub enum InspectableAttribute {
    Assignment(syn::Ident, syn::Expr),
    Tag(syn::Ident),
}
impl InspectableAttribute {
    fn ident(&self) -> &syn::Ident {
        match self {
            InspectableAttribute::Assignment(ident, _) => ident,
            InspectableAttribute::Tag(ident) => ident,
        }
    }
    pub fn is_builtin(&self) -> bool {
        let ident = self.ident();
        ident == "label" || ident == "collapse"
    }
}

fn parse_inspectable_attributes(
    input: syn::parse::ParseStream,
) -> syn::Result<impl Iterator<Item = InspectableAttribute>> {
    let parse_attribute = |input: syn::parse::ParseStream| {
        let ident: syn::Ident = input.parse()?;
        if input.peek(syn::Token![=]) {
            let _eq_token: syn::Token![=] = input.parse()?;
            let expr: syn::Expr = input.parse()?;
            Ok(InspectableAttribute::Assignment(ident, expr))
        } else if input.is_empty() {
            Ok(InspectableAttribute::Tag(ident))
        } else {
            panic!("could not parse attribute {}", ident);
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
