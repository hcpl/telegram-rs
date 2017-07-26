extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;


use std::u32;

use proc_macro::TokenStream;
use syn::{Attribute, Body, DeriveInput, Field, Ident, Lit, MetaItem, Path, StrStyle, Ty, Variant, VariantData, Visibility};


#[proc_macro_derive(SerializeIdentifiable, attributes(id))]
pub fn serialize(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_serialize_identifiable(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

enum BodyType {
    Struct,
    Enum,
}

fn impl_serialize_identifiable(ast: &DeriveInput) -> quote::Tokens {
    let item_name = &ast.ident;

    let (ext_def_item, ext_init_item) = impl_ext_item(ast);

    quote! {
        impl ::serde::Serialize for #item_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                #[derive(Serialize)]
                #ext_def_item

                let ext = #ext_init_item;

                ext.serialize(serializer)
            }
        }
    }
}

fn impl_ext_item(ast: &DeriveInput) -> (DeriveInput, quote::Tokens) {
    match ast.body {
        Body::Struct(ref data) => {
            let (ext_attrs, ext_def_data, ext_init_data) =
                impl_ext_attrs_data(BodyType::Struct, &ast.attrs, data);

            let ext_def_item = DeriveInput {
                ident: Ident::from("Extended"),
                vis: Visibility::Inherited,
                attrs: ext_attrs,
                generics: ast.generics.clone(),
                body: Body::Struct(ext_def_data),
            };

            let ext_init_item = quote! {
                Extended {
                    #ext_init_data
                }
            };

            (ext_def_item, ext_init_item)
        },

        Body::Enum(ref variants) => {
            let item_name = &ast.ident;

            let mut ext_def_variants = Vec::new();
            let mut ext_init_variants = Vec::new();

            for variant in variants {
                let (ext_attrs, ext_def_data, ext_init_data) =
                    impl_ext_attrs_data(BodyType::Enum, &variant.attrs, &variant.data);

                if variant.discriminant.is_some() {
                    unreachable!();
                }

                let variant_name = &variant.ident;
                let ext_fields_names = ext_def_data.fields()
                    .iter()
                    .map(|field| field.ident.clone().unwrap())
                    .filter(|field_name| field_name != "_id")
                    .collect::<Vec<_>>();
                let ext_def_variant = Variant {
                    ident: variant_name.clone(),
                    attrs: ext_attrs,
                    data: ext_def_data,
                    discriminant: None,
                };

                ext_def_variants.push(ext_def_variant);
                ext_init_variants.push(quote! {
                    #item_name::#variant_name { #(#ext_fields_names),* } => Extended::#variant_name {
                        #ext_init_data
                    },
                });
            }

            let ext_def_item = DeriveInput {
                ident: Ident::from("Extended"),
                vis: Visibility::Inherited,
                attrs: ast.attrs.clone(),
                generics: ast.generics.clone(),
                body: Body::Enum(ext_def_variants),
            };

            let ext_init_item = quote! {
                match *self {
                    #(#ext_init_variants)*
                }
            };

            (ext_def_item, ext_init_item)
        },
    }
}

fn impl_ext_attrs_data(body_type: BodyType,
                       attrs: &[Attribute],
                       data: &VariantData)
                      -> (Vec<Attribute>, VariantData, quote::Tokens) {
    let mut id = None;
    let mut ext_attrs = Vec::new();

    for attr in attrs {
        if let MetaItem::NameValue(ref id_ident, Lit::Str(ref id_str, StrStyle::Cooked)) = attr.value {
            if id_ident.as_ref() == "id" {
                let id_value = u32::from_str_radix(&id_str[2..], 16).unwrap();
                id = Some(id_value);
                continue;
            }
        }

        ext_attrs.push(attr.clone());
    }

    if id.is_none() {
        unreachable!();
    }

    let (ext_def_data, ext_init_data) = impl_ext_fields(body_type, data, id.unwrap());

    (ext_attrs, ext_def_data, ext_init_data)
}

fn impl_ext_fields(body_type: BodyType, data: &VariantData, id: u32) -> (VariantData, quote::Tokens) {
    let id_def_field = Field {
        ident: Some(Ident::from("_id")),
        vis: Visibility::Inherited,
        attrs: Vec::new(),
        ty: Ty::Path(None, Path::from("u32")),
    };

    let mut ext_def_fields = vec![id_def_field];
    let mut ext_init_fields = quote! { _id: #id, };

    match *data {
        VariantData::Struct(ref def_fields) => {
            ext_def_fields.extend(def_fields.clone());

            for def_field in def_fields {
                if let Some(ref name) = def_field.ident {
                    ext_init_fields.append(match body_type {
                        BodyType::Struct => quote! { #name: self.#name, },
                        BodyType::Enum   => quote! { #name: #name, },
                    });
                } else {
                    unreachable!()
                }
            }
        },
        VariantData::Tuple(_)               => unreachable!(),
        VariantData::Unit                   => { /* Do nothing */ },
    }

    (VariantData::Struct(ext_def_fields), ext_init_fields)
}
