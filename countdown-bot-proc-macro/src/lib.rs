use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Field, Ident, Variant};
#[proc_macro_attribute]
pub fn impl_cq_tostring(item: TokenStream, input: TokenStream) -> TokenStream {
    let cqtype = parse_macro_input!(item as syn::Ident);
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_data = match &ast.data {
        Data::Struct(e) => e,
        _ => {
            return quote_spanned! {
                ast.span() => compile_error!("Expected struct");
            }
            .into()
        }
    };
    let fields = struct_data
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<&Ident>>();
    let name = &ast.ident;
    let output = quote! {
        #ast
        impl ToString for #name {
            fn to_string(&self) -> String {
                let mut out = String::from("[CQ:");
                out.push_str(stringify!(#cqtype));
                #(out.push(',');out.push_str(stringify!(#fields));out.push('=');out.push_str(self. #fields .to_string().as_str());)*
                out.push(']');
                return out;
            }
        }
    };
    return output.into();
}
#[proc_macro_attribute]
pub fn impl_upcast(item: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let trait_ident = parse_macro_input!(item as syn::Ident);

    let enum_data = match &ast.data {
        Data::Enum(e) => e,
        _ => {
            return quote_spanned! {
                ast.span() => compile_error!("Expected enum");
            }
            .into()
        }
    };
    let variants = enum_data.variants.iter().collect::<Vec<&Variant>>();
    let variant_ident = variants
        .iter()
        .filter(|s| s.ident != "Unknown")
        .map(|s| s.ident.clone())
        .collect::<Vec<Ident>>();
    let mut unknown_found = false;
    for one in variants.iter() {
        if one.ident == "Unknown" {
            unknown_found = true;
        } else {
            let fields = one.fields.iter().collect::<Vec<&Field>>();
            if fields.len() != 1 {
                return quote_spanned! {
                    one.span() => compile_error!("Expected exactly one field!");
                }
                .into();
            }
        }
    }
    if !unknown_found {
        return quote_spanned! {
            ast.span() => compile_error!("Expected a variant called 'Unknown'!");
        }
        .into();
    }
    let name = &ast.ident;
    let output = quote! {
        #ast
        impl #name {
            pub fn to_event_trait(self) -> std::sync::Arc<dyn #trait_ident>{
                return match self {
                    Self::Unknown => UnknownEvent::get_instance(),
                    #(Self::#variant_ident(evt) => std::sync::Arc::new(evt) as std::sync::Arc<dyn #trait_ident>),*
                };
            }
        }
    };
    return output.into();
}
