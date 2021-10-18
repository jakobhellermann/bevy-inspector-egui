use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

#[derive(Default)]
pub struct InspectableContainerAttributes {
    pub generics: Option<syn::WhereClause>,
}

pub fn inspectable_container_attributes(
    attrs: &[syn::Attribute],
) -> syn::Result<InspectableContainerAttributes> {
    let attrs = attrs
        .iter()
        .filter(|attr| attr.path.get_ident().map_or(false, |p| p == "inspectable"))
        .map(|attr| attr.parse_meta());

    let mut all = InspectableContainerAttributes::default();

    for attr in attrs {
        let list = match attr? {
            syn::Meta::List(list) => list,
            meta => {
                return Err(syn::Error::new(
                    meta.span(),
                    "invalid attribute, expected #[inspectable(...)]",
                ))
            }
        };
        for attr in list.nested {
            let meta = match attr {
                syn::NestedMeta::Meta(meta) => meta,
                other => {
                    return Err(syn::Error::new(
                        other.span(),
                        "invalid attribute, expected #[inspectable(options)",
                    ))
                }
            };

            match meta {
                syn::Meta::NameValue(name_value)
                    if name_value
                        .path
                        .get_ident()
                        .map_or(false, |i| i == "override_where_clause") =>
                {
                    let where_clause = match &name_value.lit {
                        syn::Lit::Str(str) => {
                            let where_clause = format!("where {}", str.value());
                            let where_clause: syn::WhereClause = syn::parse_str(&where_clause)
                                .map_err(|e| {
                                    syn::Error::new(
                                        str.span(),
                                        format!("failed to parse where clause: {}", e),
                                    )
                                })?;
                            where_clause
                        }
                        other => {
                            return Err(syn::Error::new(other.span(), "expected string literal"))
                        }
                    };
                    all.generics = Some(where_clause);
                }
                other => return Err(syn::Error::new(other.span(), "invalid attribute")),
            }
        }
    }

    Ok(all)
}

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

    pub fn is_field_builtin(&self) -> bool {
        let ident = match self.lhs() {
            syn::Member::Named(ident) => ident,
            syn::Member::Unnamed(_) => return false,
        };

        ident == "label"
            || ident == "collapse"
            || ident == "default"
            || ident == "ignore"
            || ident == "wrapper"
            || ident == "read_only"
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
) -> syn::Result<Vec<InspectableAttribute>> {
    Ok(attrs
        .iter()
        .filter(|attr| attr.path.get_ident().map_or(false, |p| p == "inspectable"))
        .map(|attr| attr.parse_args_with(parse_inspectable_attributes))
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect())
}

#[derive(Default)]
pub struct InspectableFieldAttributes {
    pub collapse: bool,
    pub label: Option<String>,
    pub default: Option<syn::Expr>,
    pub ignore: bool,
    pub read_only: bool,
    pub wrapper: Option<syn::ExprPath>,
    pub custom_attributes: Vec<InspectableAttribute>,
}

impl InspectableFieldAttributes {
    pub fn create_options_struct(&self, ty: &syn::Type) -> TokenStream {
        let fields = self.custom_attributes.iter().map(|attribute| {
            let name = attribute.lhs();
            let value = attribute.rhs();
            quote! { options.#name = std::convert::From::from(#value); }
        });

        quote! {
            {
                let mut options = <#ty as bevy_inspector_egui::Inspectable>::Attributes::default();
                #(#fields)*
                options
            }
        }
    }

    pub fn label<'a>(&'a self, fallback: &'a str) -> &'a str {
        self.label.as_deref().unwrap_or(fallback)
    }

    pub fn decorate_ui(&self, mut ui: TokenStream, collapse_label: &str, i: usize) -> TokenStream {
        if self.collapse {
            ui = quote! { bevy_inspector_egui::egui::CollapsingHeader::new(#collapse_label).id_source(#i as u64).show(ui, |ui| { #ui }); };
        }
        if self.read_only {
            ui = quote! { ui.scope(|ui| { ui.set_enabled(false); #ui }); };
        }
        if let Some(wrapper_fn) = &self.wrapper {
            ui = quote! { #wrapper_fn(ui, |ui| { #ui }); };
        }

        ui
    }
}

pub fn inspectable_field_attributes(
    attrs: &[syn::Attribute],
) -> syn::Result<InspectableFieldAttributes> {
    let mut all = InspectableFieldAttributes::default();

    let (builtin_attributes, custom_attributes): (Vec<_>, Vec<_>) =
        extract_inspectable_attributes(attrs)?
            .into_iter()
            .partition(InspectableAttribute::is_field_builtin);

    // builtins
    for builtin_attribute in builtin_attributes {
        match builtin_attribute {
            InspectableAttribute::Tag(syn::Member::Named(ident)) if ident == "collapse" => {
                all.collapse = true;
            }
            InspectableAttribute::Tag(syn::Member::Named(ident)) if ident == "ignore" => {
                all.ignore = true;
            }
            InspectableAttribute::Tag(syn::Member::Named(ident)) if ident == "read_only" => {
                all.read_only = true;
            }
            #[rustfmt::skip]
            InspectableAttribute::Assignment(syn::Member::Named(ident), expr) if ident == "label" => {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(str), .. }) = expr {
                    all.label = Some(str.value());
                } else {
                    return Err(syn::Error::new(expr.span(), "label needs to be a string literal"))
                };
            }
            #[rustfmt::skip]
            InspectableAttribute::Assignment(syn::Member::Named(ident), expr) if ident == "default" => {
                all.default = Some(expr);
            }
            #[rustfmt::skip]
            InspectableAttribute::Assignment(syn::Member::Named(ident), expr) if ident == "wrapper"=> {
                let path = match expr {
                    syn::Expr::Path(path) => path,
                    _ => return Err(syn::Error::new(expr.span(), "`wrapper` attribute expected a path to a function"))
                };
                all.wrapper = Some(path);
            }
            InspectableAttribute::Tag(name) | InspectableAttribute::Assignment(name, _) => {
                match name {
                    syn::Member::Named(name) => {
                        return Err(syn::Error::new(name.span(), "unknown attribute"))
                    }
                    syn::Member::Unnamed(_) => unreachable!(),
                }
            }
        }
    }

    all.custom_attributes = custom_attributes;
    Ok(all)
}
