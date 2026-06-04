


use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Attribute, Data, Ident, LitStr, Variant, spanned::Spanned};


// mod __private {
//     pub use commun_utils_handler;
// }

// macro_rules! regex_patterns {
//     ($($pat:expr),* $(,)?) => {
//         const COUNT: usize = <[()]>::len(&[$(regex_patterns!(@sub $pat)),*]);
//     };

//     (@sub $x:expr) => (());
// }

#[proc_macro_derive(FileScanner,attributes(regex))]
pub fn file_scanner_derive(input: TokenStream) -> TokenStream {
    let file_scanner_to_deriver = syn::parse(input).unwrap();
    match impl_file_scanner_macro(&file_scanner_to_deriver) {
        Ok(token) => token ,
        Err(err_token) =>  err_token
    }
}

#[proc_macro_derive(IterableStringifyEnum,attributes(into))]
pub fn iterable_enum_macro_derive(input: TokenStream) -> TokenStream {
    let enum_deriver = syn::parse(input).unwrap();
    match impl_iterable_enum_macro(&enum_deriver) {
        Ok(token) => token ,
        Err(err_token) =>  err_token
    }
}



fn impl_iterable_enum_macro(ast: &syn::DeriveInput) -> Result<TokenStream,TokenStream> {
    if let Data::Enum(data_enum) = &ast.data {
        // let (mut variants,mut regex):(Vec<&Ident>,Vec<LitStr>) = (Vec::new(),Vec::new());
        // let variants:Vec<&Ident> = data_enum.variants.iter().map(|i| &i.ident).collect();
        let mut variants:Vec<&Ident> = Vec::new();
        let mut value:Vec<LitStr> = Vec::new();

        for variant in data_enum.variants.iter() {
            if !variant.attrs.is_empty() {
                let into = syn::Ident::new("into",variant.span());
                handle_single_attr(variant, &into, &mut |attr|{
                    let meta:LitStr = attr.parse_args().map_err(|error|{
                        error.to_compile_error()
                    })?;
                    variants.push(&variant.ident);
                    value.push(meta);
                    Ok(())
                })?
                // let mut attrs =  variant.attrs.iter();
                // if let (Some(attr),None) = (attrs.next(),attrs.next()) {
                //     if attr.path().is_ident("to") {
                //         let meta:LitStr = attr.parse_args().map_err(|error|{
                //             error.to_compile_error()
                //         })?;
                //         variants.push(&variant.ident);
                //         value.push(meta);
                //     } else {
                //         let message = "the only attribute accepted is regex \n correction : #[regex(...)]\n".to_string() + &variant.ident.to_string();
                //         return Err(syn::Error::new_spanned(
                //             variant,
                //             message
                //         )
                //         .to_compile_error()
                //         .into());
                //     }
                // } else {
                //     let message = "Can't only be one attribute for the FileScanner variants \n correction : #[regex(...)]\n".to_string() + &variant.ident.to_string();
                //     return Err(syn::Error::new_spanned(
                //     variant,
                //         message
                //     )
                //     .to_compile_error()
                //     .into());
                // }
            } else {
                let lit_str = LitStr::new(
                    &variant.ident.to_string(),
                    variant.ident.span(),
                );
                variants.push(&variant.ident);
                value.push(lit_str);
            }
        }

        // for variant in data_enum.variants.iter() {
        //     if variant.attrs.is_empty() {
        //         let mut iter = variant.attrs.iter(); 
        //         if let (Some(attr),None) = (iter.next(),iter.next()) {
        //             if attr.path().is_ident("to") {
        //             let meta:LitStr = attr.parse_args().map_err(|error|{
        //                 error.to_compile_error()
        //             })?;
        //             variants.push(&variant.ident);
        //             value.push(meta);
        //             } else {
        //                 return Err(syn::Error::new_spanned(attr,"attribute not valid (use to)").to_compile_error().into());
        //             }
        //         } else {
        //             return Err(syn::Error::new_spanned(
        //             variant,
        //                 "you must use only one attribute"
        //             )
        //             .to_compile_error()
        //             .into());
        //         }
        //     } else {
        //         let lit_str:LitStr = syn::parse_str(&variant.ident.to_string()).map_err(|err|{
        //             syn::Error::new_spanned(
        //                     variant,
        //                     err.to_string()
        //                 )
        //                 .to_compile_error()
        //         })?;
        //         variants.push(&variant.ident);
        //         value.push(lit_str);
        //     }
        // }
        let name = &ast.ident;
        
        let generation = quote! {
            // use std::str::FromStr;
            // use commun_utils_handler::IterableStringifyEnum;
            // mod __macro_imports {
            //     pub use commun_utils_handler::IterableStringifyEnum;
            // }
            impl commun_utils_handler::IterableStringifyEnum for #name
            {
                fn iter_enum()-> Vec<#name>
                {
                    vec![#(#name::#variants,)*]
                }
            }

            impl From<#name> for String {
            fn from(value: #name) -> Self {
                    match value {
                        #(#name::#variants => String::from(#value),)* 
                    }
                }
            }

            impl std::str::FromStr for #name {
                type Err = String;
                fn from_str(value: &str) -> Result<Self, Self::Err> {
                    match value {
                        #(x if x == String::from(#value) => Ok(#name::#variants),)* 
                        _ => Err(String::from(value))
                    }
                }
            }
        };
        Ok(generation.into())
    } else {
        return Err(syn::Error::new_spanned(
                ast,
                "IterableEnum can only be used on enums"
            )
            .to_compile_error()
            .into());
    }
}

fn handle_single_attr(variant: &Variant,ident_name:&Ident,func:&mut impl FnMut(&Attribute)->Result<(),TokenStream>)-> Result<(),TokenStream>{
    let mut attrs =  variant.attrs.iter();
    if let (Some(attr),None) = (attrs.next(),attrs.next()) {
        if attr.path().is_ident(ident_name) {
            func(attr)?;
            Ok(())
        } else {
            let message = "unknown attribute".to_string() + &attr.path().get_ident().unwrap().to_string() + "\n correction: #[" + "(...)]\n" + &variant.ident.to_string();
            return Err(syn::Error::new_spanned(
                variant,
                message
            )
            .to_compile_error()
            .into());
        }
    } else {
        return Err(syn::Error::new_spanned(
        variant,
            "you must use only one attribute"
        )
        .to_compile_error()
        .into());
    }
}

fn impl_file_scanner_macro(ast: &syn::DeriveInput) -> Result<TokenStream,TokenStream> {
    if let Data::Enum(data_enum) = &ast.data {
        let (mut variants,mut regex):(Vec<&Ident>,Vec<LitStr>) = (Vec::new(),Vec::new());
        for variant in data_enum.variants.iter() {
            if !variant.attrs.is_empty() {
                let regex_ident = syn::Ident::new("regex", variant.span());
                handle_single_attr(variant, &regex_ident, &mut |attr|{
                    let meta:LitStr = attr.parse_args().map_err(|error|{
                        error.to_compile_error()
                    })?;
                    variants.push(&variant.ident);
                    regex.push(meta);
                    Ok(())
                })?;
                // let mut attrs =  variant.attrs.iter();
                // if let (Some(attr),None) = (attrs.next(),attrs.next()) {
                //     if attr.path().is_ident("regex") {
                        
                //     } else {
                //         let message = "the only attribute accepted is regex \n correction : #[regex(...)]\n".to_string() + &variant.ident.to_string();
                //         return Err(syn::Error::new_spanned(
                //             variant,
                //             message
                //         )
                //         .to_compile_error()
                //         .into());
                //     }
                // } else {
                //     let message = "Can't only be one attribute for the FileScanner variants \n correction : #[regex(...)]\n".to_string() + &variant.ident.to_string();
                //     return Err(syn::Error::new_spanned(
                //     variant,
                //         message
                //     )
                //     .to_compile_error()
                //     .into());
                // }
            }
        }
        let name = &ast.ident;
        let len_regex:usize = variants.len();
        let generation = quote! {
            impl FileScanner for #name
            {
                fn scanner<'scanner>()->ScanBytesSubject<'scanner>
                {
                    const LEN_REGEX:usize = #len_regex;
                    let variants:[&str;LEN_REGEX] = [#(stringify!(#variants),)*];
                    let regexes:[&str;LEN_REGEX] = [#(#regex,)*];
                    
                    ScanBytesSubject::new::<LEN_REGEX>(variants,regexes).unwrap()
                }
            }
        };
        Ok(generation.into())
    } else {
        return Err(syn::Error::new_spanned(
                ast,
                "FileScanner can only be used on enums"
            )
            .to_compile_error()
            .into());
    }
} 


// #[proc_macro_derive(FileScanner)]
