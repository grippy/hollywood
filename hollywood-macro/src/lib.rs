use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn;
use syn::spanned::Spanned;

fn get_version_variant(ty: String) -> syn::Result<proc_macro2::TokenStream> {
	let version_ty = format_ident!("{}", ty);
	let code = quote! {
		#version_ty::VERSION => {
			let msg = #version_ty::from_bytes(bytes)?;
			match dispatch_type {
				&DispatchType::Send => {
					let result = <Self as Handle<#version_ty>>::send(self, msg).await;
					return match result {
						Ok(_) => Ok((None, None)),
						Err(err) => Err(err.into()),
					};
				}
				&DispatchType::Request => {
					let result = <Self as Handle<#version_ty>>::request(self, msg).await;
					return match result {
						Ok(Some(msg)) => {
							Ok((Some(#version_ty::version()), Some(msg.into_bytes()?)))
						},
						Ok(None) => Ok((Some(#version_ty::version()), None)),
						Err(err) => Err(err.into()),
					};
				}
				&DispatchType::Subscribe => {
					let result = <Self as Handle<#version_ty>>::subscribe(self, msg).await;
					return match result {
						Ok(_) => Ok((None, None)),
						Err(err) => Err(err.into()),
					};
				}
			}
		},
	};
	Ok(code)
}

// construct dispatch type for each item
fn get_version_vec(ty: String) -> syn::Result<proc_macro2::TokenStream> {
	let version_ty = format_ident!("{}", ty);
	let code = quote! {
		#version_ty::dispatch_type()
	};
	Ok(code)
}

/// impl_hollywood_dispatch
fn impl_hollywood_dispatch(
	input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
	let derive_input: syn::DeriveInput = syn::parse2(input)?;
	let attr = &derive_input
		.attrs
		.iter()
		.find(|attr| attr.path.is_ident("dispatch"))
		.ok_or_else(|| {
			syn::Error::new(derive_input.span(), "dispatch(...) attribute is required")
		})?;

	if attr.tokens.is_empty() {
		return Err(syn::Error::new(
			attr.span(),
			"dispatch(...) attribute requires parenthesis",
		));
	};

	let meta = &attr.parse_meta()?;
	let list = match meta {
		syn::Meta::List(list) => list,
		_ => {
			return Err(syn::Error::new_spanned(
				attr,
				"dispatch attribute arguments must be a list",
			))
		}
	};

	let mut msg_types = Vec::new();
	for item in list.nested.iter() {
		match item {
			// parse type token
			syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
				msg_types.push(path.get_ident().unwrap().to_string());
			}
			// parse "type" token
			syn::NestedMeta::Lit(syn::Lit::Str(lit_str)) => {
				msg_types.push(lit_str.value());
			}
			_ => {}
		}
	}

	if msg_types.len() == 0 {
		return Err(syn::Error::new_spanned(attr, "missing dispatch msg type"));
	}

	// struct ident + generics, type generics, and where clause
	let ident = &derive_input.ident;
	let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

	// dispatch fn
	let version_arms = &msg_types
		.iter()
		.map(|attr| get_version_variant(attr.to_string()))
		.collect::<syn::Result<Vec<_>>>()
		.unwrap();

	let dispatch_fn = quote! {
		async fn dispatch(
			&mut self,
			version: String,
			dispatch_type: &DispatchType,
			bytes: &Vec<u8>,
		) -> Result<DispatchResponse> {
			use log::debug;
			debug!("[DISPATCH] type: {:?}, version: {:?}, len: {}", &dispatch_type, &version, &bytes.len());
			match &version[..] {
				#(#version_arms)*
				&_ => panic!("dispatch found an unknown version variant"),
			}
		}
	};

	// dispatch_types fn
	let version_items = &msg_types
		.iter()
		.map(|attr| get_version_vec(attr.to_string()))
		.collect::<syn::Result<Vec<_>>>()
		.unwrap();

	let dispatch_types_fn = quote! {
		fn instance_dispatch_types(&self) -> Vec<String> {
			Self::dispatch_types()
		}
		fn dispatch_types() -> Vec<String> {
			vec![#(#version_items),*]
		}
	};

	let code = quote! {
		#[automatically_derived]
		#[async_trait]
		impl #impl_generics Dispatch for #ident #ty_generics #where_clause {
			#dispatch_types_fn
			#dispatch_fn
		}
	};
	Ok(code)
}

fn impl_hollywood_actor_mailbox(
	input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
	let derive_input: syn::DeriveInput = syn::parse2(input)?;
	let ident = &derive_input.ident;
	let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

	let code = quote! {

		#[automatically_derived]
		#[async_trait]
		impl #impl_generics hollywood::ActorMailbox for #ident #ty_generics #where_clause {
			async fn mailbox<M: Msg>(
				system_name: String,
				nats_uri: String,
			) -> Result<hollywood::mailbox::Mailbox> {
				hollywood::mailbox::Mailbox::new::<#ident, M>(system_name, nats_uri).await
			}
			async fn mailbox_from_env<M: Msg>() -> Result<hollywood::mailbox::Mailbox> {
				hollywood::mailbox::Mailbox::from_env::<#ident, M>().await
			}
		}

	};
	Ok(code)
}

fn impl_hollywood(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let mut gen = proc_macro2::TokenStream::new();
	gen.extend(impl_hollywood_dispatch(input.clone())?);
	gen.extend(impl_hollywood_actor_mailbox(input.clone())?);
	Ok(gen)
}

#[proc_macro_derive(Hollywood, attributes(dispatch))]
pub fn hollywood_derive(input: TokenStream) -> TokenStream {
	let gen = impl_hollywood(input.into());
	gen.unwrap_or_else(|e| e.to_compile_error()).into()
}
