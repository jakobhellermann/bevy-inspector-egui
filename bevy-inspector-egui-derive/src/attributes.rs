fn parse_inspectable_attributes(
    input: syn::parse::ParseStream,
) -> syn::Result<impl Iterator<Item = (syn::Ident, syn::Expr)>> {
    let parse_attribute = |input: syn::parse::ParseStream| {
        let ident: syn::Ident = input.parse()?;
        let _eq_token: syn::Token![=] = input.parse()?;
        let expr: syn::Expr = input.parse()?;
        Ok((ident, expr))
    };

    input
        .parse_terminated::<_, syn::Token![,]>(parse_attribute)
        .map(IntoIterator::into_iter)
}

/// extracts [(min, 8), (field, vec2(1.0, 1.0))] from `#[inspectable(min = 8, field = vec2(1.0, 1.0))]`,
pub fn inspectable_attributes(
    attrs: &[syn::Attribute],
) -> impl Iterator<Item = (syn::Ident, syn::Expr)> + '_ {
    attrs
        .iter()
        .filter(|attr| attr.path.get_ident().map_or(false, |p| p == "inspectable"))
        .flat_map(|attr| attr.parse_args_with(parse_inspectable_attributes).unwrap())
}
