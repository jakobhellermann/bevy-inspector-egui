use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use std::borrow::Cow;

use crate::{
    attributes::{inspectable_field_attributes, InspectableFieldAttributes},
    utils,
};

pub fn expand_enum(
    derive_input: &syn::DeriveInput,
    data: &syn::DataEnum,
) -> syn::Result<TokenStream> {
    let name = &derive_input.ident;

    let container_attributes =
        crate::attributes::inspectable_container_attributes(&derive_input.attrs)?;

    let variant_names: Vec<_> = data.variants.iter().map(|variant| &variant.ident).collect();

    // used to check whether the combobox and the fields below should be `ui.group`ed,
    // which is the case if the variant contains any fields.
    let should_group_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let value = !variant.fields.is_empty();
        quote! { Self::#variant_name { .. } => #value, }
    });

    let variants: Vec<_> = enum_variants(data)
        .map(|(variant, fields)| {
            let fields: Vec<_> = fields
                .into_iter()
                .map(|(i, field)| {
                    let member = utils::field_accessor(field, i);
                    let attributes = inspectable_field_attributes(&field.attrs)?;
                    Ok((i, field, member, attributes))
                })
                .collect::<syn::Result<_>>()?;
            Ok((variant, fields))
        })
        .collect::<syn::Result<_>>()?;

    let ui_match_arms = variants.iter().map(|(variant, fields)| {
        let initializers = fields.iter().map(|(_, _, member, options)| {
            let value = match &options.default {
                Some(expr) => expr.to_token_stream(),
                None => quote! { Default::default() },
            };
            quote! {
                #member: #value,
            }
        });

        let set_variant = quote! {
            if let Self::#variant { .. } = self {} else {
                *self = Self::#variant {
                    #(#initializers)*
                }
            }
        };
        let field_ui = (!fields.is_empty()).then(|| field_ui(variant, fields));
        quote! {
            stringify!(#variant) => {
                #set_variant
                #field_ui
            },
        }
    });

    let egui = quote! { bevy_inspector_egui::egui };

    let generic_for_impl =
        utils::with_inspectable_bound(&derive_input.generics, &container_attributes.generics);
    let (impl_generics, ty_generics, where_clause) = generic_for_impl.split_for_impl();
    let where_clause = container_attributes.generics.as_ref().or(where_clause);

    Ok(quote! {
        impl #impl_generics bevy_inspector_egui::Inspectable for #name #ty_generics #where_clause {
            type Attributes = ();


            fn ui(&mut self, ui: &mut #egui::Ui, options: Self::Attributes, context: &bevy_inspector_egui::Context) -> bool {
                let mut variant = match self {
                    #(Self::#variant_names { .. } => stringify!(#variant_names),)*
                };

                let should_group = match self {
                    #(#should_group_arms)*
                };
                fn group_if(ui: &mut #egui::Ui, val: bool, mut f: impl FnMut(&mut #egui::Ui)) {
                    if val { ui.group(f); } else { f(ui); }
                }

                let mut changed = false;

                group_if(ui, should_group, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            bevy_inspector_egui::egui::ComboBox::from_id_source(context.id())
                                .selected_text(variant)
                                .show_ui(ui, |ui| {
                                    #(if ui.selectable_label(matches!(self, #name::#variant_names { .. }), stringify!(#variant_names)).clicked() {
                                        variant = stringify!(#variant_names);
                                        changed = true;
                                    })*
                                });
                        });

                        match variant {
                            #(#ui_match_arms)*
                            _ => {},
                        };
                    });
                });

                changed
            }
        }
    })
}

fn enum_variants<'a>(
    data: &'a syn::DataEnum,
) -> impl Iterator<Item = (&'a syn::Ident, Vec<(usize, &'a syn::Field)>)> {
    data.variants.iter().map(|variant| {
        let has_inspectable_attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path.get_ident().map_or(false, |p| p == "inspectable"))
            .is_some();
        assert!(
            !has_inspectable_attr,
            "inspectable attributes are not supported on enum variants."
        );

        let fields = match &variant.fields {
            syn::Fields::Named(fields) => Some(fields.named.iter()),
            syn::Fields::Unnamed(fields) => Some(fields.unnamed.iter()),
            syn::Fields::Unit => None,
        };
        let fields = fields.into_iter().flatten().enumerate().collect();
        (&variant.ident, fields)
    })
}

// Example:
// if let Self::A { a: a, b: b } = self {
//     ui.horizontal(|ui| {
//         ui.label(stringify!(a));
//         let options = <#ty as Inspectable>::Attributes::default();
//         a.ui(ui, options, context);
//     });
//     ui.horizontal(|ui| {
//         ui.label(stringify!(b));
//         let options = <#ty as Inspectable>::Attributes::default();
//         b.ui(ui, options, context);
//     });
// }
fn field_ui(
    variant: &syn::Ident,
    f: &[(usize, &syn::Field, syn::Member, InspectableFieldAttributes)],
) -> TokenStream {
    let field_names = f.iter().map(|(i, field, _, _)| name_for_member(field, *i));
    let members = f.iter().map(|(_, _, m, _)| m);

    let field_ui = f.iter().map(|(i, field, member, attributes)| {
        let binding_name = name_for_member(field, *i);
        let options = attributes.create_options_struct(&field.ty);

        let field_label = utils::field_label(field, *i);
        let field_label = attributes.label(&field_label);

        if attributes.ignore {
            return quote! {};
        }

        let ui = if f.len() == 1 {
            quote! {
                let options = #options;
                changed |= #binding_name.ui(ui, options, context);
            }
        } else {
            quote! {
                ui.horizontal(|ui| {
                    ui.label(stringify!(#member));
                    let options = #options;
                    changed |= #binding_name.ui(ui, options, context);
                });
            }
        };

        let ui = attributes.decorate_ui(ui, field_label, *i);
        ui
    });

    quote! {
        ui.separator();
        #[allow(non_shorthand_field_patterns, unused_variables)]
        if let Self::#variant { #(#members: #field_names),* } = self {
            #(#field_ui)*
        }
    }
}

fn name_for_member(field: &syn::Field, i: usize) -> Cow<'_, syn::Ident> {
    match &field.ident {
        Some(name) => Cow::Borrowed(name),
        None => {
            let name = syn::Ident::new(&format!("field_{}", i), Span::call_site());
            Cow::Owned(name)
        }
    }
}
