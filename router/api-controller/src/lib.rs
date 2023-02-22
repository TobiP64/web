// MIT License
//
// Copyright (c) 2019-2023 Tobias Pfeiffer
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![feature(proc_macro_diagnostic, proc_macro_span)]

extern crate proc_macro;

use quote::ToTokens;
use syn::spanned::Spanned;
use {std::{fmt::Write, str::FromStr}, proc_macro::{Span}, proc_macro2::*, syn::*, quote::quote};

#[allow(clippy::manual_map)]
fn get_attr(attrs: &mut Vec<Attribute>, name: &str) -> Option<Attribute> {
	match attrs.iter()
		.enumerate()
		.find(|(_, attr)| attr.path.is_ident(
			&syn::Ident::new(name, proc_macro2::Span::call_site())))
	{
		Some((i, _)) => Some(attrs.remove(i)),
		None         => None
	}
}

fn parse_attr(attr: &Attribute) -> Option<Group> {
	let mut tokens = attr.tokens.clone().into_iter();
	
	let group = match tokens.next() {
		Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => group,
		Some(tt) => {
			tt.span().unwrap().error("failed to parse attribute: expected (...)".to_string()).emit();
			return None;
		}
		_ => return None
	};
	
	if let Some(tt) = tokens.next() {
		tokens
			.fold(tt.span(), |span, tt| span.join(tt.span()).unwrap())
			.unwrap().error("failed to parse attribute: unexpected tokens".to_string())
			.emit();
		return None;
	}
	
	Some(group)
}

fn parse_route(tokens: TokenStream) -> Option<TokenStream> {
	let (mut method, mut path) = (TokenStream::from_str("_").unwrap(), TokenStream::from_str("_").unwrap());
	let mut tokens = tokens.into_iter();
	
	let group = match tokens.next() {
		Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => group,
		Some(tt) => {
			tt.span().unwrap().error("failed to parse attribute: expected (...)".to_string()).emit();
			return None;
		}
		None => return Some(quote! { (_, _) })
	};
	
	if let Some(tt) = tokens.next() {
		tokens
			.fold(tt.span(), |span, tt| span.join(tt.span()).unwrap())
			.unwrap().error("failed to parse attribute: unexpected tokens".to_string())
			.emit();
	}
	
	let mut tokens = group.stream().into_iter();
	
	while let Some(tt) = tokens.next() {
		let ident = match tt {
			TokenTree::Ident(ident) => ident,
			tt =>  {
				tt.span().unwrap().error("failed to parse attribute: expected identifier".to_string()).emit();
				return None;
			}
		};
		
		match ident.to_string().as_str() {
			"method" => {
				match tokens.next() {
					Some(TokenTree::Punct(v)) if v.as_char() == '=' => (),
					Some(tt) => {
						tt.span().unwrap().error("failed to parse attribute: expected assignment".to_string()).emit();
						return None;
					}
					None => {
						group.span_close().unwrap().error("failed to parse attribute: expected assignment".to_string()).emit();
						return None;
					}
				}
				
				match tokens.next() {
					Some(TokenTree::Literal(v)) => {
						let v = v.to_string();
						let v = v.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
						let v = v[..1].to_ascii_uppercase() + &v[1..].to_ascii_lowercase();
						method = TokenStream::from_str(&format!("::net::http::Method::{}", v)).unwrap();
					}
					Some(tt) => {
						tt.span().unwrap().error("failed to parse attribute: expected string literal".to_string()).emit();
						return None;
					}
					None => {
						group.span_close().unwrap().error("failed to parse attribute: expected string literal".to_string()).emit();
						return None;
					}
				}
			}
			"path"   => {
				match tokens.next() {
					Some(TokenTree::Punct(v)) if v.as_char() == '=' => (),
					Some(tt) => {
						tt.span().unwrap().error("failed to parse attribute: expected assignment".to_string()).emit();
						return None;
					}
					None => {
						group.span_close().unwrap().error("failed to parse attribute: expected assignment".to_string()).emit();
						return None;
					}
				}
				
				match tokens.next() {
					Some(TokenTree::Literal(v)) => {
						let v = v.to_string();
						let input = v.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
						let mut buf = "[".to_string();
						
						for s in input.split('/') {
							match s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
								Some(s) => match s.strip_suffix("...") {
									Some(s) => write!(&mut buf, "{} @ .., ", s).unwrap(),
									None    => write!(&mut buf, "{}, ", s).unwrap()
								},
								None    => write!(&mut buf, "\"{}\", ", s).unwrap()
							}
						}
						
						buf.push(']');
						path = TokenStream::from_str(&buf).unwrap();
					}
					Some(tt) => {
						tt.span().unwrap().error("failed to parse attribute: expected string literal".to_string()).emit();
						return None;
					}
					None => {
						group.span_close().unwrap().error("failed to parse attribute: expected string literal".to_string()).emit();
						return None;
					}
				}
			}
			_ => {
				ident.span().unwrap().error("invalid attribute: expected `method` or `path`".to_string()).emit();
				return None;
			}
		}
		
		match tokens.next() {
			None => break,
			Some(TokenTree::Punct(v)) if v.as_char() == ',' => continue,
			Some(tt) => {
				tt.span().unwrap().error("failed to parse attribute: expected `,` or nothing".to_string()).emit();
				return None;
			}
		}
	}
	
	Some(quote! { (#method, #path) })
}

fn parse_error(tokens: TokenStream) -> Option<TokenStream> {
	let mut tokens = tokens.into_iter();
	
	let group = match tokens.next() {
		Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => group,
		Some(tt) => {
			tt.span().unwrap().error("failed to parse attribute: expected (...)".to_string()).emit();
			return None;
		}
		None => return Some(quote! { Err(e) })
	};
	
	if let Some(tt) = tokens.next() {
		tokens
			.fold(tt.span(), |span, tt| span.join(tt.span()).unwrap())
			.unwrap().error("failed to parse attribute: unexpected tokens".to_string())
			.emit();
	}
	
	let tokens = group.stream();
	
	if tokens.to_string() == "panic" {
		Some(quote! { Err(e) })
	} else {
		Some(quote! { Err(e) if e.is::<#tokens>() })
	}
}

fn parse_input(ty: &mut PatType) -> Option<TokenStream> {
	for (i, attr) in ty.attrs.iter_mut().enumerate() {
		let v = match &attr.path.get_ident().map(Ident::to_string).as_deref() {
			Some("path")  => {
				let ident = &ty.pat;
				quote! { std::convert::TryInto::try_into(PathArg(#ident).get())? }
			}
			Some("query") => {
				let group = parse_attr(attr)?;
				let mut tokens = group.stream().into_iter();
				
				match tokens.next() {
					Some(TokenTree::Literal(v)) => {
						if ty.ty.to_token_stream().to_string().starts_with("Option") {
							quote! {
								{ query.get(#v)
									.map(|v| std::convert::TryInto::try_into(*v)).transpose()? }
							}
						} else {
							quote! {
								{ query.get(#v)
									.map(|v| std::convert::TryInto::try_into(*v)).transpose()?
									.ok_or_else(|| ::net_services::Error::new("required query parameter not present"))? }
							}
						}
					}
					Some(tt) => {
						tt.span().unwrap().error("failed to parse attribute: expected string literal".to_string()).emit();
						return None;
					}
					None => {
						group.span_close().unwrap().error("failed to parse attribute: expected string literal".to_string()).emit();
						return None;
					}
				}
			}
			Some("header") => {
				let group = parse_attr(attr)?;
				let mut tokens = group.stream().into_iter();
				
				match tokens.next() {
					Some(TokenTree::Ident(v)) => {
						if ty.ty.to_token_stream().to_string().starts_with("Option") {
							quote! { headers.iter().find_map(|v| match v {
								::net::http::Header::#v(v) => Some(v),
								_ => None
							}) }
						} else {
							quote! { headers.iter().find_map(|v| match v {
								::net::http::Header::#v(v) => Some(v),
								_ => None
							}).ok_or_else(|| ::net_services::Error::new("required header not present"))? }
						}
					}
					Some(tt) => {
						tt.span().unwrap().error("failed to parse attribute: expected identifier".to_string()).emit();
						return None;
					}
					None => {
						group.span_close().unwrap().error("failed to parse attribute: expected identifier".to_string()).emit();
						return None;
					}
				}
			}
			Some("body") => quote! { {
				let mut buf = Vec::with_capacity(0x1000);
				stream.read_to_end(&mut buf).await?;
				std::convert::TryInto::try_into(buf)?
			} },
			Some("stream") => quote! { (&mut*stream) },
			_ => continue
		};
		
		let _ = ty.attrs.remove(i);
		return Some(v);
	}
	
	ty.span().unwrap().error("argument not annotated".to_string()).emit();
	None
}

//fn parse_output(_ty: &mut ReturnType) -> Option<TokenStream> {
//	unimplemented!()
//}

#[proc_macro_attribute]
pub fn controller(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let wrapper_name = match syn::parse::<syn::Ident>(attr) {
		Ok(v) => v,
		Err(e) => {
			Span::call_site().error(format!("failed to parse attribute: {}", e)).emit();
			std::process::exit(0);
		}
	};
	
	let mut item_impl = match syn::parse::<syn::ItemImpl>(item) {
		Ok(v) => v,
		Err(e) => {
			Span::call_site().error(format!("failed to parse impl-block: {}", e)).emit();
			std::process::exit(0);
		}
	};
	
	let routes = item_impl.items.iter_mut().filter_map(|item| {
		let (attr, sig) = match item {
			ImplItem::Method(method) => (get_attr(&mut method.attrs, "route")?, &mut method.sig),
			_ => return None
		};
		
		let fn_name = &sig.ident;
		let pattern = parse_route(attr.tokens)?;
		let input   = sig.inputs.iter_mut().filter_map(|input| match input {
			FnArg::Typed(input) => Some(parse_input(input)?),
			FnArg::Receiver(_) => None
		});
		//let output  = parse_output(&mut sig.output)?;
		
		Some(quote! { #pattern => self_.0.#fn_name(#(#input),*).await })
	}).collect::<Vec<_>>();
	
	let errors = item_impl.items.iter_mut().filter_map(|item| {
		let (attr, sig) = match item {
			ImplItem::Method(method) => (get_attr(&mut method.attrs, "error")?, &mut method.sig),
			_ => return None
		};
		
		let fn_name = &sig.ident;
		let pattern = parse_error(attr.tokens)?;
		Some(quote! { #pattern => self.0.#fn_name(&headers, stream, e).await })
	}).collect::<Vec<_>>();
	
	let context_ty = &*item_impl.self_ty;
	
	proc_macro::TokenStream::from(quote! {
		#item_impl
		
		#[allow(non_camel_case_types)]
		struct #wrapper_name(::std::sync::Arc<#context_ty>);
		
		impl ::net_services::interfaces::StreamHandler<dyn ::net::http::traits::AsyncStream> for #wrapper_name {
			fn accept<'a>(&'a self, stream: &'static mut dyn ::net::http::traits::AsyncStream) ->
				::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::net_services::Result<()>> + Send + 'a>>
			{
				::std::boxed::Box::pin(async move {
					use ::net::http::Method::*;
					let mut headers = ::net::http::traits::AsyncStreamExt::read_headers(stream).await?;
					smol::io::AsyncReadExt::read(stream, &mut []).await?;
				
					let method = headers.iter()
						.find_map(::net::http::Header::as_method)
						.unwrap();
					
					let path = headers.iter()
						.find_map(::net::http::Header::as_path)
						.unwrap();
				
					let (path, query) = path.split_once('?')
						.unwrap_or((path, ""));
				
					let query = query.split('&')
						.map(|v| v.split_once('=').unwrap_or((v, "")))
						.collect::<::std::collections::HashMap<_, _>>();
				
					let path = str::split(path, '/')
						.collect::<::std::vec::Vec<_>>();
					
					async fn __try(
						self_:   &#wrapper_name,
						stream:  &mut dyn ::net::http::traits::AsyncStream,
						headers: &[::net::http::Header],
						method:  &::net::http::Method,
						path:    ::std::vec::Vec<&str>,
						query:   ::std::collections::HashMap<&str, &str>
					) -> ::net_services::Result<()> {
						pub struct PathArg<T>(T);
					
						impl<'a> PathArg<&'a &'a str> {
							fn get(self) -> &'a str {
								*self.0
							}
						}
					
						impl<'a> PathArg<&'a [&'a str]> {
							fn get(self) -> &'a [&'a str] {
								self.0
							}
						}
					
						match (method, &*path) {
							#(#routes),*
						}
					}
				
					match __try(self, stream, &headers, method, path, query).await {
						#(#errors, )*
						r => r
					}
				})
			}
		}
	})
}