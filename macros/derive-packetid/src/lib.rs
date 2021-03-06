use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, AttrStyle, DeriveInput, Ident, Lit, LitInt, Token,
};

#[derive(Debug)]
struct Args {
    ident: String,
    id: LitInt,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let _: Token!(=) = input.parse()?;
        let id: Lit = input.parse()?;
        if ident != "id" {
            return Err(syn::Error::new(
                ident.span(),
                format!("expected `id` but got {}", ident),
            ));
        }
        let id = if let Lit::Int(v) = id {
            v
        } else {
            let e = syn::Error::new(ident.span(), "Expected u16");
            return Err(e);
        };
        let args = Self {
            ident: ident.to_string(),
            id,
        };
        Ok(args)
    }
}

fn derive_packet_id(input: DeriveInput) -> syn::Result<TokenStream> {
    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let generics = input.generics;
    let attr = input
        .attrs
        .iter()
        .find(|a| a.style == AttrStyle::Outer && a.path.is_ident("packet"))
        .ok_or_else( ||
            syn::Error::new(name.span(),"Missing Packet id! please add #[packet(id = ..)] on the struct"),
        )?;
    let args: Args = attr.parse_args()?;
    let id = args.id;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #impl_generics tq_network::PacketID for #name #ty_generics #where_clause {
            fn id() -> u16 { #id as u16 }
        }
    };
    Ok(expanded.into())
}

#[proc_macro_derive(PacketID, attributes(packet))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    derive_packet_id(input).unwrap_or_else(|err| err.to_compile_error().into())
}
