extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Visibility};

#[proc_macro_derive(UIEditable)]
pub fn derive_ui_edit(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let name = input.ident;
    let idents = match input.data {
        Data::Struct(data) => data.fields.into_iter().filter_map(|field| match field.vis {
            Visibility::Public(_) => Some(field.ident).flatten(),
            _ => None,
        }),
        _ => panic!("UIEdit derive macro is only intended for structs!"),
    };

    let mut implementation = quote! {};
    for ident in idents {
        let this = quote! {
            let position = position + Vector2::new(0.0, size.y * 1.2);
            self.#ident.draw_edit(position, size, stringify!(#ident));
        };

        implementation = quote! {
            #implementation
            #this
        };
    }

    quote! {
        impl UIEdit for #name {
            fn draw_edit(&mut self, position: Vector2<f32>, size: Vector2<f32>, label: &str) {
                #implementation
            }
        }
    }
    .into()
}
