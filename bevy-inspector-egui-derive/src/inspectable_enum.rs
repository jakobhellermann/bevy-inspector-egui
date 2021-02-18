use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_enum(derive_input: &syn::DeriveInput, data: &syn::DataEnum) -> TokenStream {
    let name = &derive_input.ident;

    let variant_names: Vec<_> = data.variants.iter().map(|variant| &variant.ident).collect();

    // used to check whether the combobox and the fields below should be `ui.group`ed,
    // which is the case if the variant contains any fields.
    let should_group_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let value = !variant.fields.is_empty();
        quote! { Self::#variant_name { .. } => #value, }
    });

    let ui_match_arms = enum_variants(data).map(|(variant, fields)| {
        let set_variant = quote! {
            if let Self::#variant { .. } = self {} else {
                *self = Self::#variant {
                    #(#fields: Default::default()),*
                }
            }
        };
        let field_ui = (!fields.is_empty()).then(|| field_ui(variant, &fields));

        quote! {
            stringify!(#variant) => {
                #set_variant
                #field_ui
            },
        }
    });
    let egui = quote! { bevy_inspector_egui::egui };

    quote! {
        impl bevy_inspector_egui::Inspectable for #name {
            type Attributes = ();


            fn ui(&mut self, ui: &mut #egui::Ui, options: Self::Attributes, context: &bevy_inspector_egui::Context) {
                let mut variant = match self {
                    #(Self::#variant_names { ..} => stringify!(#variant_names),)*
                };

                let should_group = match self {
                    #(#should_group_arms)*
                };
                fn group_if(ui: &mut #egui::Ui, val: bool, mut f: impl FnMut(&mut #egui::Ui)) {
                    if val { ui.group(f) } else { f(ui) }
                }

                group_if(ui, should_group, |ui| {
                    ui.vertical(|ui| {
                        bevy_inspector_egui::egui::combo_box(ui, context.id(), variant, |ui| {
                            #(if ui.selectable_label(matches!(self, #name::#variant_names { .. }), stringify!(#variant_names)).clicked() {
                                variant = stringify!(#variant_names);
                            })*
                        });

                        match variant {
                            #(#ui_match_arms)*
                            _ => {},
                        };
                    });
                });
            }
        }
    }
}

fn enum_variants(data: &syn::DataEnum) -> impl Iterator<Item = (&syn::Ident, Vec<syn::Member>)> {
    data.variants.iter().map(|variant| {
        let fields = match &variant.fields {
            syn::Fields::Named(fields) => Some(fields.named.iter()),
            syn::Fields::Unnamed(fields) => Some(fields.unnamed.iter()),
            syn::Fields::Unit => None,
        };
        let fields = fields.into_iter().flatten();
        let fields: Vec<_> = fields
            .enumerate()
            .map(|(i, field)| match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None => syn::Member::Unnamed(syn::Index::from(i)),
            })
            .collect();

        (&variant.ident, fields)
    })
}

// Example:
// if let Self::A { a: a, b: b } = self {
//     ui.horizontal(|ui| {
//         ui.label(stringify!(a));
//         a.ui(ui, Default::default(), context);
//     });
//     ui.horizontal(|ui| {
//         ui.label(stringify!(b));
//         b.ui(ui, Default::default(), context);
//     });
// }
fn field_ui(variant: &syn::Ident, fields: &[syn::Member]) -> TokenStream {
    let field_names: Vec<_> = fields
        .iter()
        .map(|member| match member {
            syn::Member::Named(ident) => quote! { #ident },
            syn::Member::Unnamed(syn::Index { index, span }) => {
                let name = syn::Ident::new(&format!("field_{}", index), *span);
                quote! { #name }
            }
        })
        .collect();

    quote! {
        ui.separator();
        #[allow(non_shorthand_field_patterns)]
        if let Self::#variant { #(#fields: #field_names),* } = self {
            #(ui.horizontal(|ui| {
                ui.label(stringify!(#fields));
                #field_names.ui(ui, Default::default(), context);
            });)*
        }
    }
}
