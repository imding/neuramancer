use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DeriveInput},
};

#[proc_macro_derive(PutId, attributes(put_ids))]
pub fn derive_record_id_string(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("RecordIdString can only be derived for structs"),
    };

    let nested_updates = fields.iter().filter_map(|field| {
        let has_nested_attr = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("put_ids"));

        if !has_nested_attr {
            return None;
        }

        let field_name = field.ident.as_ref().unwrap();

        Some(quote! {
            for item in self.#field_name.iter_mut() {
                item.put_id();
            }
        })
    });

    let expanded = quote! {
        #[cfg(feature = "server")]
        impl SurrealRecord for #name {
            fn put_id(&mut self) {
                self.id_ = Some(self.id.key().to_string());
                #(#nested_updates)*
            }
        }
    };

    TokenStream::from(expanded)
}
