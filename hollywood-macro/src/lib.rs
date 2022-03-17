use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn;

fn get_version_variant(ty: String) -> syn::Result<proc_macro2::TokenStream> {
	let version_ty = format_ident!("{}", ty);
	let code = quote! {
		#version_ty::VERSION => {
			let msg = #version_ty::from_bytes(bytes)?;
			match dispatch {
				Dispatch::Send => {
					let result = <Self as Handle<#version_ty>>::send(self, msg);
					return match result {
						Ok(_) => Ok(None),
						Err(err) => Err(err.into()),
					};
				}
				Dispatch::Request => {
					let result = <Self as Handle<#version_ty>>::request(self, msg);
					return match result {
						Ok(Some(msg)) => Ok(Some(msg.into_bytes()?)),
						Ok(None) => Ok(None),
						Err(err) => Err(err.into()),
					};
				}
				Dispatch::Subscribe => {
					let result = <Self as Handle<#version_ty>>::subscribe(self, msg);
					return match result {
						Ok(_) => Ok(None),
						Err(err) => Err(err.into()),
					};
				}
			}
		}
	};
	Ok(code)
}

/// `dispatch` attribute macro provides a default implementation
/// for dispatching versioned messages to the appropriate
/// actor Handler<Msg> implementation
#[proc_macro_attribute]
pub fn dispatch(attr: TokenStream, _item: TokenStream) -> TokenStream {
	// println!("attr: \"{}\"", attr.to_string());
	// println!("item: \"{}\"", item.to_string());
	let version_arms = attr
		.into_iter()
		.filter(|attr| match attr {
			proc_macro::TokenTree::Ident(_) => true,
			_ => false,
		})
		.map(|attr| get_version_variant(attr.to_string()))
		.collect::<syn::Result<Vec<_>>>()
		.unwrap();

	let code = quote! {
		fn dispatch(
			&mut self,
			version: &'static str,
			dispatch: Dispatch,
			bytes: &Vec<u8>,
		) -> Result<Option<Vec<u8>>> {
			match version {
				#(#version_arms)*
				&_ => panic!("dispatch found an unknown version variant"),
			}
		}
	};
	// println!("dispatch_fn: {}", code.to_string());
	code.into()
}

fn get_version_vec(ty: String) -> syn::Result<proc_macro2::TokenStream> {
	let version_ty = format_ident!("{}", ty);
	let code = quote! {
		#version_ty::dispatch_type()
	};
	Ok(code)
}

/// `dispatch_types` attribute macro generates Actor dispatch_types
/// based on the provided Msg attributes.
#[proc_macro_attribute]
pub fn dispatch_types(attr: TokenStream, _item: TokenStream) -> TokenStream {
	let version_items = attr
		.into_iter()
		.filter(|attr| match attr {
			proc_macro::TokenTree::Ident(_) => true,
			_ => false,
		})
		.map(|attr| get_version_vec(attr.to_string()))
		.collect::<syn::Result<Vec<_>>>()
		.unwrap();

	let code = quote! {
		fn dispatch_types() -> Vec<String> {
			vec!(#(#version_items),*)
		}
	};
	code.into()
}
