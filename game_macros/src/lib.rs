extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident, Visibility};

fn prettify_ident(ident: &Ident) -> String {
    let string = ident
        .to_string()
        .split("_")
        .map(|part| {
            part.char_indices()
                .map(|(i, c)| {
                    if i == 0 {
                        c.to_uppercase().next().unwrap()
                    } else {
                        c
                    }
                })
                .fold(String::new(), |mut acc, c| {
                    acc.push(c);
                    acc
                })
        })
        .fold(String::new(), |acc, x| acc + " " + x.as_str())
        .trim()
        .to_string();

    string
}

#[proc_macro_derive(UIEditable, attributes(display_as, gap_after))]
pub fn derive_ui_edit(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => data.fields.into_iter().filter_map(|field| match field.vis {
            Visibility::Public(_) => {
                let mut display_as = None;
                let mut gap_after = None;
                for attr in field.attrs {
                    if attr.path().is_ident("display_as") {
                        if let Ok(meta) = attr.meta.require_list() {
                            display_as = Some(meta.tokens.clone());
                        }
                    }
                    if attr.path().is_ident("gap_after") {
                        if let Ok(meta) = attr.meta.require_list() {
                            gap_after = Some(meta.tokens.clone());
                        }
                    }
                }

                if let Some(ident) = field.ident {
                    Some((ident, display_as, gap_after))
                } else {
                    None
                }
            }
            _ => None,
        }),
        _ => panic!("UIEditable derive macro is only intended for structs!"),
    };

    let text_offset = 1.0f32;
    let mut implementation = quote! {
        let mut total_size = if !label.is_empty() {
            draw_text(
                label,
                position.x,
                position.y + FONT_SIZE_MEDIUM * #text_offset,
                FONT_SIZE_MEDIUM,
                Color::rgb(0, 0, 0).as_mq(),
            );
            Vector2::new(0.0, FONT_SIZE_MEDIUM * (0.5 + #text_offset))
        } else {
            Vector2::new(0.0, 0.0)
        };
    };
    for (ident, display_as, gap_after) in fields {
        let label = if let Some(display_as) = display_as {
            display_as.to_string().replace("\"", "")
        } else {
            prettify_ident(&ident)
        };

        let gap_after = gap_after.unwrap_or(quote! {Vector2::new(0.0, 0.0)});

        let this = quote! {
            let this_position = position + total_size;
            total_size.y += self.#ident.draw_edit(this_position, input_size, #label).y;
            total_size += Vector2::new(0.0, input_size.y * 0.2) + #gap_after;
        };

        implementation = quote! {
            #implementation
            #this
        };
    }

    quote! {
        impl UIEdit for #name {
            fn draw_edit(&mut self, position: Vector2<f32>, input_size: Vector2<f32>, label: &str) -> Vector2<f32> {
                #implementation

                total_size + Vector2::new(0.0, input_size.y)
            }
        }
    }
    .into()
}
