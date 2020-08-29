use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, AttrStyle, DeriveInput, Ident, Lit, Token,
};

#[derive(Debug)]
struct Args {
    ident: Ident,
    token: Token![=],
    id: Lit,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Self {
            ident: input.parse()?,
            token: input.parse()?,
            id: input.parse()?,
        };
        if args.ident != "id" {
            return Err(syn::Error::new(
                args.ident.span(),
                format!("expected `id` but got {}", args.ident),
            ));
        }
        if let Lit::Int(_) = &args.id {
            Ok(args)
        } else {
            let e = syn::Error::new(args.ident.span(), "Expected u16");
            Err(e)
        }
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
        impl #impl_generics network::PacketID for #name #ty_generics #where_clause {
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
