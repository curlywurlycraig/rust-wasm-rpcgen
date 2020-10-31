use proc_macro::TokenStream;
use syn::{ItemFn};
use quote::quote;

use crate::utils::{get_input_args_as_pat_tuple};

pub fn impl_frontend_remote(ast: ItemFn) -> TokenStream {
    let endpoint = "http://localhost:3030/rpc"; // TODO read from config

    let ident = &ast.sig.ident;
    let fn_args = &ast.sig.inputs;
    let return_type = &ast.sig.output;
    let input_as_tuple = get_input_args_as_pat_tuple(&fn_args);

    let return_statement = match return_type {
        syn::ReturnType::Default => quote! { () },
        _ => quote! { bincode::deserialize(&bin[..]).unwrap() }
    };

    let gen = quote! {
        use wasm_bindgen::prelude::*;

        pub async fn #ident(#fn_args) #return_type {
            let window = web_sys::window().unwrap();

            // This actually should be called in some init function.
            // It also should be disabled in production to make the wasm smaller.
            console_error_panic_hook::set_once();

            // Serialize input
            let input_as_bytes: Vec<u8> = bincode::serialize(&#input_as_tuple).unwrap();

            // Create request
            let mut opts = web_sys::RequestInit::new();
            opts.method("POST");
            opts.mode(web_sys::RequestMode::Cors);
            opts.body(Some(&Uint8Array::from(&input_as_bytes[..])));

            let request = web_sys::Request::new_with_str_and_init(#endpoint, &opts).unwrap();

            let response: web_sys::Response = wasm_bindgen_futures::JsFuture::from(window
                .fetch_with_request(&request)).await
                .unwrap()
                .dyn_into()
                .expect("Failed to cast to response.");

            let bin: Vec<u8> = wasm_bindgen_futures:: JsFuture::from(response.text().unwrap()).await.unwrap()
                .as_string().unwrap().into_bytes();

            #return_statement
        }
    };

    gen.into()
}



// TODO Make it be async
// TODO Error handling