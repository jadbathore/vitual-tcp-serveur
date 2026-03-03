use proc_macro::TokenStream;

use quote::quote;
use syn;


#[proc_macro_derive(FileScanner)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Construit une représentation du code Rust en arborescence
    // syntaxique que nous pouvons manipuler
    let file_scanner_to_deriver = syn::parse(input).unwrap();

    // Construit l'implémentation du trait
    impl_hello_macro(&file_scanner_to_deriver)
}


fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    for attr in ast.attrs.iter() { 
        if attr.path().is_ident("regex") {
            let meta = attr.parse_args::<syn::LitStr>().unwrap();
            println!("Regex = {}", meta.value());
        }
    }
    let nom = &ast.ident;
    let generation = quote! {
        impl FileScanner for #nom {
            fn scan() {
                println!("Hello, Macro ! Mon nom est {}", stringify!(#nom));
            }
        }

        
    };
    generation.into()
}