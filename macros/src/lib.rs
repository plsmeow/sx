use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, ExprLit, Lit, Variant};

#[proc_macro_derive(Index)]
pub fn derive_indexed_enum(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let ident = &input.ident;

  let variants = match input.data {
    Data::Enum(e) => e.variants,
    _ => {
      return syn::Error::new_spanned(ident, "unsupported type")
        .to_compile_error()
        .into();
    }
  };

  let mut vars_from = Vec::new();
  let mut vars_to = Vec::new();

  for v in variants.into_iter() {
    let (v_ident, v_value) = match extract_index(&v) {
      Ok(x) => x,
      Err(e) => return e.to_compile_error().into(),
    };

    vars_from.push(quote! {
      #v_value => Some(#ident::#v_ident),
    });

    vars_to.push(quote! {
      Self::#v_ident => #v_value,
    });
  }

  let expanded = quote! {
    impl salarixi_extensions::index::IndexExt for #ident {
      fn from_index(index: u8) -> Option<Self> {
        match index {
          #(#vars_from)*
          _ => None,
        }
      }
    }
  };

  expanded.into()
}

fn extract_index(v: &Variant) -> Result<(syn::Ident, u8), syn::Error> {
  let ident = v.ident.clone();

  let (_, discr) = v
    .discriminant
    .as_ref()
    .ok_or_else(|| syn::Error::new_spanned(v, "index must be specified using `=` and be of type `u8`"))?;

  let index = extract_index_from_expr(discr)?;
  Ok((ident, index))
}

fn extract_index_from_expr(expr: &Expr) -> Result<u8, syn::Error> {
  match expr {
    Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) => i
      .base10_parse::<u8>()
      .map_err(|_| syn::Error::new_spanned(expr, "index must be of type `u8`")),
    _ => Err(syn::Error::new_spanned(
      expr,
      "index must be specified using `=` and be of type `u8`",
    )),
  }
}
