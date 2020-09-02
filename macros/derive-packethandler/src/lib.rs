use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, AttrStyle, Data, DataEnum, DeriveInput, Expr, Ident,
    Token,
};

#[derive(Debug)]
struct Args {
    state: Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let _: Token!(=) = input.parse()?;
        let state: Expr = input.parse()?;
        if ident != "state" {
            return Err(syn::Error::new(
                ident.span(),
                format!("expected `state` but got {}", ident),
            ));
        }
        let args = Self { state };
        Ok(args)
    }
}
fn derive_packet_handler(input: DeriveInput) -> syn::Result<TokenStream> {
    let body = if let Data::Enum(e) = input.data {
        body(e)?
    } else {
        return Err(syn::Error::new(
            input.ident.span(),
            "You can only derive `PacketHandler` for enums",
        ));
    };
    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let attr = input
        .attrs
        .iter()
        .find(|a| a.style == AttrStyle::Outer && a.path.is_ident("handle"))
        .ok_or_else( ||
            syn::Error::new(name.span(),"Missing ActorState! please add #[handle(state = ..)] on the enum"),
        )?;
    let args: Args = attr.parse_args()?;
    let state = args.state;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #[async_trait::async_trait]
        impl #impl_generics tq_network::PacketHandler for #name #ty_generics #where_clause {
            type Error = crate::Error;
            type ActorState = #state;
             async fn handle(
                 (id, bytes): (u16, bytes::Bytes),
                 actor: &tq_network::Actor<Self::ActorState>,
                ) -> Result<(), Self::Error> {
                    use tq_network::{PacketID, PacketProcess};
                    #body
                    Ok(())
                }
        }
    };
    Ok(expanded.into())
}

fn body(e: DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    let vars: Vec<_> = e
        .variants
        .into_iter()
        .filter(|v| v.fields.is_empty())
        .collect();
    let arms = vars.into_iter().map(|v| {
        let ident = v.ident;
        quote! {
            _ if id == #ident::id() => {
                let msg = <#ident as tq_network::PacketDecode>::decode(&bytes)?;
                tracing::debug!("{:?}", msg);
                msg.process(actor).await?;
            },
        }
    });
    let tokens = quote! {
        match id {
            #(#arms)*
            _ => {
                tracing::warn!("Got Unknown Packet, id = {}", id);
            }
        };
    };
    Ok(tokens)
}

#[proc_macro_derive(PacketHandler, attributes(handle))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    derive_packet_handler(input)
        .unwrap_or_else(|err| err.to_compile_error().into())
}
