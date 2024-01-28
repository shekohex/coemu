use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, ReturnType};

struct Args {
    msg: Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let msg: Ident = input.parse()?;
        let args = Self { msg };
        Ok(args)
    }
}

macro_rules! syn_assert {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err(syn::Error::new_spanned($cond, $msg));
        }
    };
}

fn derive_packet_processor(args: Args, inner_fn: ItemFn) -> syn::Result<TokenStream> {
    let msg_ty = args.msg;
    // make sure the function has the right signature
    // fn process(msg: msg_ty, actor: &Resource<ActorHandle>) -> Result<(),
    // crate::Error>
    let fn_sig = &inner_fn.sig;
    syn_assert!(fn_sig.constness.is_none(), "const fn not supported");
    syn_assert!(fn_sig.asyncness.is_none(), "async fn not supported");
    syn_assert!(fn_sig.abi.is_none(), "abi fn not supported");
    syn_assert!(fn_sig.unsafety.is_none(), "unsafe fn not supported");
    syn_assert!(fn_sig.generics.params.is_empty(), "generic fn not supported");
    syn_assert!(fn_sig.generics.where_clause.is_none(), "generic fn not supported");
    syn_assert!(fn_sig.inputs.len() == 2, "packet processor must have two arguments");
    syn_assert!(
        fn_sig.output != ReturnType::Default,
        "packet processor must have a return type"
    );
    syn_assert!(
        matches!(fn_sig.inputs[0], FnArg::Typed(_)),
        "packet processor must have a typed msg argument"
    );
    syn_assert!(
        matches!(fn_sig.inputs[1], FnArg::Typed(_)),
        "packet processor must have a typed actor argument"
    );
    match fn_sig.inputs[0] {
        FnArg::Receiver(_) => unreachable!(),
        FnArg::Typed(ref p) => syn_assert!(
            p.ty == syn::parse_quote!(#msg_ty),
            "packet processor must have a typed msg argument"
        ),
    };
    match fn_sig.inputs[1] {
        FnArg::Receiver(_) => unreachable!(),
        FnArg::Typed(ref p) => {
            syn_assert!(
                p.ty == syn::parse_quote!(&Resource<ActorHandle>),
                "packet processor must have a typed actor argument"
            );
        },
    };

    let inner_fn_call = inner_fn.sig.ident.clone();
    let msg_ty_name = msg_ty.to_string().to_lowercase();
    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #[export_name = "__alloc"]
        #[cfg(target_arch = "wasm32")]
        pub extern "C" fn __alloc(size: u32) -> *mut u8 {
            #[cfg(not(feature = "std"))]
            let v = ::alloc::vec::Vec::with_capacity(size as usize);
            #[cfg(feature = "std")]
            let v = ::std::vec::Vec::with_capacity(size as usize);
            let mut v = core::mem::ManuallyDrop::new(v);
            unsafe {
                v.set_len(size as usize);
            }
            v.shrink_to_fit();
            let ptr = v.as_mut_ptr();
            ptr
        }


        #inner_fn

        #[::tq_bindings::externref(crate = "tq_bindings::anyref")]
        #[export_name = "process_packet"]
        #[cfg(target_arch = "wasm32")]
        pub unsafe extern "C" fn _process(
            packet_ptr: *mut u8,
            packet_len: u32,
            actor: &::tq_bindings::Resource<ActorHandle>,
        ) -> i32 {
            ::tq_bindings::set_panic_hook_once(#msg_ty_name);
            ::tq_bindings::setup_logging(#msg_ty_name);
            #[cfg(not(feature = "std"))]
            let packet = ::alloc::vec::Vec::from_raw_parts(packet_ptr, packet_len as _, packet_len as _);
            #[cfg(feature = "std")]
            let packet = ::std::vec::Vec::from_raw_parts(packet_ptr, packet_len as _, packet_len as _);
            let bytes = ::bytes::BytesMut::from(packet.as_slice()).freeze();
            let packet = match <#msg_ty as ::tq_network::PacketDecode>::decode(&bytes) {
                Ok(packet) => packet,
                Err(e) => {
                    tracing::error!(error = ?e, "While decoding the packet");
                    return 0xdec0de;
                }
            };
            match #inner_fn_call(packet, actor) {
                Ok(()) => 0,
                Err(e) => {
                    tracing::error!(error = ?e, "While handling the packet");
                    0x00f
                }
            }
        }
    };
    Ok(expanded.into())
}

#[proc_macro_attribute]
pub fn packet_processor(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let args = parse_macro_input!(args as Args);
    derive_packet_processor(args, input).unwrap_or_else(|err| err.to_compile_error().into())
}
