use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn derive_server(input: DeriveInput) -> syn::Result<TokenStream> {
    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #impl_generics network::Server for #name #ty_generics #where_clause {}
    };
    Ok(expanded.into())
}

#[proc_macro_derive(Server)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    derive_server(input).unwrap_or_else(|err| err.to_compile_error().into())
}
