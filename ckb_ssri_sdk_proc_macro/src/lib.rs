#![no_std]

extern crate alloc;
extern crate proc_macro;

use core::panic;

use ckb_hash::blake2b_256;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::ImplItemFn;
use syn::{
    parse::Parse, parse_macro_input, Expr, ExprLit, Ident, ImplItem, ItemFn, ItemImpl, Lit, Meta,
    Token,
};

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

fn encode_u64_vector(val: impl AsRef<[u64]>) -> Vec<u8> {
    let val = val.as_ref();
    u32::to_le_bytes(val.len() as u32)
        .into_iter()
        .chain(val.iter().flat_map(|v| u64::to_le_bytes(*v)))
        .collect()
}

// Struct to hold method metadata for reflection and dispatch
struct SSRIMethodMetadata {
    pub method_name: String,
    pub namespace: String,
    pub method_signature: String,
    pub method_attributes: SSRIMethodAttributes,
}

#[derive(Debug)]
enum SSRIMethodLevel {
    Code,
    Script,
    Cell,
    Transaction,
}

impl Default for SSRIMethodLevel {
    fn default() -> Self {
        SSRIMethodLevel::Code
    }
}

#[derive(Debug)]
struct SSRIMethodAttributes {
    pub implemented: bool,
    pub internal: bool,
    pub transaction: bool,
    pub level: SSRIMethodLevel,
}

impl Default for SSRIMethodAttributes {
    fn default() -> Self {
        SSRIMethodAttributes {
            implemented: true,
            internal: false,
            transaction: false,
            level: SSRIMethodLevel::Code,
        }
    }
}

impl Parse for SSRIMethodAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

#[derive(Debug)]
struct SSRIModuleAttributes {
    pub version: String,
    pub base: Option<String>,
}

impl Default for SSRIModuleAttributes {
    fn default() -> Self {
        SSRIModuleAttributes {
            version: (&"0").to_string(),
            base: None,
        }
    }
}

impl Parse for SSRIModuleAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

#[derive(Debug)]
enum SSRISDKProcMacroError {
    InvalidMethodAttribute,
    InvalidModuleAttribute,
    InvalidTraitName,
}

// Function to extract the trait name (used as the namespace if `base` is not provided)
fn extract_trait_name(impl_block: &ItemImpl) -> Result<String, SSRISDKProcMacroError> {
    if let Some((_, path, _)) = &impl_block.trait_ {
        if let Some(segment) = path.segments.last() {
            return Ok(segment.ident.to_string());
        }
    }
    Err(SSRISDKProcMacroError::InvalidTraitName)
}

#[proc_macro_attribute]
pub fn ssri_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    let method_args = parse_macro_input!(attr as SSRIMethodAttributes);

    let method = parse_macro_input!(item as ItemFn);

    let method_metadata_const_name = format_ident!("__SSRIMETHOD_METADATA_{}", method.sig.ident);

    // Create a metadata struct to represent the parsed attributes
    let generated_method_metadata = quote! {
        const #method_metadata_const_name: SSRIMethodMetadata = SSRIMethodMetadata {
            namespace: "", // This will be set in ssri_module
            method_name: #method.sig.ident.to_string(),
            method_signature: #method.sig.to_string(),
            method_attributes: ssri_method_attributes
        };
    };

    // Return the method and the constant metadata
    let expanded = quote! {
        #method
        // #generated_method_metadata
    };

    // Return the modified method as a TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn ssri_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    let ssri_module_attributes = parse_macro_input!(attr as SSRIModuleAttributes);

    let trait_name = extract_trait_name(&input).unwrap();

    // Determine namespace based on `base` or fall back to the trait name
    let namespace = match ssri_module_attributes.base {
        Some(base) => base,
        None => trait_name.clone(),
    };
    // let mut dispatch_cases = Vec::new();

    // let mut module_metadata = Vec::new();

    // for item in &impl_block.items {
    //     if let ImplItemFn(method) = item {
    //         let method_name = &method.sig.ident;
    //         let method_name_str = method_name.to_string();
    //         let args_types = method.sig.inputs.iter().map(|input| {
    //             if let FnArg::Typed(pat) = input {
    //                 pat.ty.to_token_stream().to_string()
    //             } else {
    //                 panic!("Invalid method signature");
    //             }
    //         });
            
    //         // Generate a match arm for each method
    //         dispatch_cases.push(quote! {
    //             #method_name_str => self.#method_name(param),
    //         });
    //     }
    // }

    // let generated_module_metadata = quote! {
    //     const #module_metadata_const_name: &[SSRIMethodMetadata] = &[#(#module_metadata),*];
    // };

    let expanded = quote! {
        #input
        // #generated_module_metadata
    };

    TokenStream::from(expanded)
}

fn method_path(name: impl AsRef<[u8]>) -> u64 {
    u64::from_le_bytes(blake2b_256(name)[0..8].try_into().unwrap())
}

struct Methods {
    argv: Expr,
    invalid_method: Expr,
    invalid_args: Expr,
    method_keys: Vec<u64>,
    method_bodies: Vec<Expr>,
}

impl Parse for Methods {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let argv = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let invalid_method = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let invalid_args = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;

        let mut method_keys = vec![];
        let mut method_bodies = vec![];
        while !input.is_empty() {
            let name = match input.parse::<Expr>()? {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(v), ..
                }) => v.value(),
                _ => panic!("method name should be a string"),
            };
            input.parse::<Token![=>]>()?;
            let body = input.parse::<Expr>()?;
            input.parse::<Token![,]>()?;

            method_keys.push(method_path(name));
            method_bodies.push(body);
        }

        Ok(Methods {
            argv,
            invalid_method,
            invalid_args,
            method_keys,
            method_bodies,
        })
    }
}

#[proc_macro]
pub fn ssri_methods(input: TokenStream) -> TokenStream {
    let Methods {
        argv,
        invalid_method,
        invalid_args,
        method_keys,
        method_bodies,
    } = parse_macro_input!(input as Methods);

    let version_path = method_path("SSRI.version");
    let get_methods_path = method_path("SSRI.get_methods");
    let has_methods_path = method_path("SSRI.has_methods");

    let raw_methods = encode_u64_vector(
        [version_path, get_methods_path, has_methods_path]
            .iter()
            .chain(method_keys.iter())
            .copied()
            .collect::<Vec<_>>(),
    );
    let raw_methods_len = raw_methods.len();

    TokenStream::from(quote! {
        {
            use alloc::{borrow::Cow, vec::Vec};
            use ckb_std::high_level::decode_hex;
            const RAW_METHODS: [u8; #raw_methods_len] = [#(#raw_methods,)*];
            let res: Result<Cow<'static, [u8]>, Error> = match u64::from_le_bytes(
                decode_hex(&(#argv)[0])?.try_into().map_err(|_| #invalid_method)?,
            ) {
                #version_path => Ok(Cow::from(&[0][..])),
                #get_methods_path => {
                    let offset = usize::min((4 +u64::from_le_bytes(
                        decode_hex(&(#argv)[1])?
                            .try_into()
                            .map_err(|_| #invalid_args)?
                    ) as usize * 8), #raw_methods_len);
                    let limit = usize::min((4 + (offset + u64::from_le_bytes(
                        decode_hex(&(#argv)[2])?
                            .try_into()
                            .map_err(|_| #invalid_args)?
                    ) as usize) * 8), #raw_methods_len);
                    if limit == 0 {
                        Ok(Cow::from(&RAW_METHODS[offset..]))
                    } else {
                        Ok(Cow::from(&RAW_METHODS[offset..limit]))
                    }
                },
                #has_methods_path => Ok(Cow::from(
                    decode_hex(&(#argv)[1])?[4..].chunks(8).map(|path| {
                        match RAW_METHODS[4..]
                            .chunks(8)
                            .find(|v| v == &path) {
                                Some(_) => 1,
                                None => 0,
                            }
                    }).collect::<Vec<_>>()
                )),
                #(
                    #method_keys => #method_bodies,
                )*
                _ => Err(#invalid_method),
            };
            res
        }
    })
}


#[proc_macro]
pub fn ssri_wasm(input: TokenStream) -> TokenStream {
    // TODO: Export serializeStruct
    todo!()
}