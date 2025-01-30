use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Expr, Ident, Token};

#[derive(Debug, Clone)]
struct Args {
    actor_state: Expr,
    state: Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident1: Ident = input
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "expected `state` or `actor_state` but got nothing"))?;
        let _: Token!(=) = input.parse().map_err(|e| syn::Error::new(e.span(), "expected `=`"))?;
        let ident1_value: Expr = input
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "expected `Expr`"))?;
        let _: Token!(,) = input.parse().map_err(|e| syn::Error::new(e.span(), "expected `,`"))?;
        let ident2: Ident = input
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "expected `state` or `actor_state` but got nothing"))?;
        let _: Token!(=) = input.parse().map_err(|e| syn::Error::new(e.span(), "expected `=`"))?;
        let ident2_value: Expr = input
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "expected `Expr`"))?;
        let (state, actor_state) = match (ident1, ident2) {
            (ident1, ident2) if ident1 == "state" && ident2 == "actor_state" => (ident1_value, ident2_value),
            (ident1, ident2) if ident1 == "actor_state" && ident2 == "state" => (ident2_value, ident1_value),
            (v1, v2) => {
                return Err(syn::Error::new(
                    input.span(),
                    format!("expected `state` or `actor_state` but got {v1} and {v2}",),
                ))
            },
        };

        let args = Self { state, actor_state };
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
        .find(|a| a.path().is_ident("handle"))
        .ok_or_else(|| {
            syn::Error::new(
                name.span(),
                "Missing State and ActorState! please add #[handle(state = .., actor_state = ...)] on the enum",
            )
        })?;
    let args: Args = attr.parse_args()?;
    let state = args.state;
    let actor_state = args.actor_state;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #[async_trait::async_trait]
        impl #impl_generics tq_network::PacketHandler for #name #ty_generics #where_clause {
            type Error = crate::Error;
            type ActorState = #actor_state;
            type State = #state;
            #[::tracing::instrument(skip_all, fields(actor = actor.id(), packet_id = packet.0))]
             async fn handle(
                 packet: (u16, bytes::Bytes),
                 state: &Self::State,
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
    let vars = e.variants.into_iter().filter(|v| v.fields.is_empty());
    let match_stms = vars.into_iter().map(|v| {
        let ident = v.ident;
        quote! {
            #ident::PACKET_ID => {
                let maybe_msg = <#ident as tq_network::PacketDecode>::decode(&packet.1);
                match maybe_msg {
                    Ok(msg) => {
                        tracing::debug!(target: "cq_msg", "{msg:?}");
                        msg.process(state, actor).await?;
                    },
                    Err(e) => {
                        tracing::error!(id = %packet.0, error = ?e, "Failed to decode packet");
                        return Ok(());
                    }
                }
                return Ok(());
            },
        }
    });
    let tokens = quote! {
        match packet.0 {
            #(#match_stms)*
            _ => {
                tracing::warn!(id = %packet.0, "Got Unknown Packet");
            }
        }
    };
    Ok(tokens)
}

#[proc_macro_derive(PacketHandler, attributes(handle))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    derive_packet_handler(input).unwrap_or_else(|err| err.to_compile_error().into())
}
