use quote::{Tokens, ToTokens};

use codegen::{Field, TraitImpl, OuterFromImpl, Variant};
use util::{Body, VariantData};

pub struct FmiImpl<'a> {
    pub base: TraitImpl<'a>,
}

impl<'a> ToTokens for FmiImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let base = &self.base;

        let impl_block = match base.body {
            Body::Struct(VariantData::Unit) => {
                quote!(
                    fn from_word() -> ::darling::Result<Self> {
                        Ok(Self)
                    }
                )
            }
            Body::Struct(VariantData::Tuple(ref fields)) if fields.len() == 1 => {
                quote!(
                    fn from_meta_item(__item: &::syn::MetaItem) -> ::darling::Result<Self> {
                        Ok(Self(::darling::FromMetaItem::from_meta_item(__item)?))
                    }
                )
            }
            Body::Struct(VariantData::Tuple(_)) => {
                panic!("Multi-field tuples are not supported");
            }
            Body::Struct(ref data) => {
                let inits = data.fields().into_iter().map(Field::as_initializer);
                let decls = base.local_declarations();
                let core_loop = base.core_loop();
                let default = base.fallback_decl();
                let map = base.map_fn();
                

                quote!(
                    fn from_list(__items: &[::syn::NestedMetaItem]) -> ::darling::Result<Self> {
                        
                        #decls

                        #core_loop

                        #default

                        ::darling::export::Ok(Self {
                            #(#inits),*
                        }) #map
                    }
                )
            }
            Body::Enum(ref variants) => {
                let unit_arms = variants.iter().map(Variant::as_unit_match_arm);
                let struct_arms = variants.iter().map(Variant::as_data_match_arm);

                quote!(
                    fn from_list(__outer: &[::syn::NestedMetaItem]) -> ::darling::Result<Self> {
                        match __outer.len() {
                            0 => ::darling::export::Err(::darling::Error::too_few_items(1)),
                            1 => {
                                if let ::syn::NestedMetaItem::MetaItem(ref __nested) = __outer[0] {
                                    match __nested.name() {
                                        #(#struct_arms)*
                                        __other => ::darling::export::Err(::darling::Error::unknown_value(__other))
                                    }
                                } else {
                                    ::darling::export::Err(::darling::Error::unsupported_format("literal"))
                                }
                            }
                            _ => ::darling::export::Err(::darling::Error::too_many_items(1)),
                        }
                    }

                    fn from_string(lit: &str) -> ::darling::Result<Self> {
                        match lit {
                            #(#unit_arms)*
                            __other => ::darling::export::Err(::darling::Error::unknown_value(__other))
                        }
                    }
                )
            }
        };

        self.wrap(impl_block, tokens);
    }
}

impl<'a> OuterFromImpl<'a> for FmiImpl<'a> {
    fn trait_path(&self) -> Tokens {
        quote!(::darling::FromMetaItem)
    }

    fn base(&'a self) -> &'a TraitImpl<'a> {
        &self.base
    }
}