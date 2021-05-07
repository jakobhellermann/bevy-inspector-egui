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
    pub default: Option<syn::Expr>,
    pub ignore: bool,
    pub read_only: bool,
    pub wrapper: Option<syn::ExprPath>,
    pub custom_attributes: Vec<InspectableAttribute>,
}

impl InspectableAttributes {
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
            ui = quote! { ui.wrap(|ui| { ui.set_enabled(false); #ui }); };
        }
        if let Some(wrapper_fn) = &self.wrapper {
            ui = quote! { #wrapper_fn(ui, |ui| { #ui }); };
        }

        ui
    }
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
                    panic!("label needs to be a string literal");
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
                    _ => panic!("`wrapper` attribute expected a path to a function"),
                };
                all.wrapper = Some(path);
            }
            InspectableAttribute::Tag(name) | InspectableAttribute::Assignment(name, _) => {
                match name {
                    syn::Member::Named(name) => panic!("unknown attribute '{}'", name),
                    syn::Member::Unnamed(_) => unreachable!(),
                }
            }
        }
    }

    all.custom_attributes = custom_attributes;
    all
}
