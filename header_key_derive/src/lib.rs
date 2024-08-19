use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Variant};

#[proc_macro_derive(HeaderKey)]
pub fn header_key_derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse(input)
    .unwrap_or_else(|error| panic!("Unable to parse input to header_key_derive: {}", error));
  let name = &ast.ident;

  let variants = match &ast.data {
    Data::Enum(data_enum) => &data_enum.variants,
    _ => panic!("HeaderKey can only be derived for enums"),
  };

  let output = generate_match_arms(name, variants);
  println!("Generated impl: {}", output); // Debug output
  output.into()
}

fn generate_match_arms(
  name: &Ident,
  variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
) -> TokenStream {
  let match_arms = variants.iter().map(|variant| {
    let variant_name = &variant.ident;
    let variant_str = variant_name.to_string().to_case(Case::Train);

    match &variant.fields {
      Fields::Unnamed(_) => {
        // Assume this is the Custom variant
        quote! {
            Self::#variant_name(ref s) => s,
        }
      }
      _ => {
        quote! {
            Self::#variant_name => #variant_str,
        }
      }
    }
  });

  quote! {
      impl AsRef<str> for #name {
          fn as_ref(&self) -> &str {
              match self {
                  #(#match_arms)*
              }
          }
      }
  }
  .into()
}
