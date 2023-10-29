#[proc_macro_derive(Object)]
pub fn object_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_macro(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn impl_macro(ast: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !matches!(ast.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new_spanned(ast, "this derive macro only works on structs"));
    }

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let traits = quote::quote! {
        #[automatically_derived]
        impl #impl_generics crate::Requestable for #name #ty_generics #where_clause {
            fn auth(&self) -> Option<crate::Authorization> {
                self.auth.clone()
            }

            fn set_auth(&mut self, auth: Option<crate::Authorization>) {
                self.auth = auth;
            }
        }

        #[automatically_derived]
        impl #impl_generics crate::Xmlable for #name #ty_generics #where_clause {
            fn url(&self) -> &str {
                &self.url
            }
        }

        #[automatically_derived]
        impl #impl_generics crate::Children for #name #ty_generics #where_clause {
            fn new<S>(url: S) -> Self
            where
                S: Into<String>,
            {
                Self {
                    url: url.into(),
                    auth: None,
                }
            }
        }
    };

    Ok(traits)
}
