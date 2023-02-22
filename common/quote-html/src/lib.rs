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

#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use {std::{fmt::Write, collections::VecDeque}, ::proc_macro::*, quote::quote};

macro_rules! emit {
    ($span:expr, $( $tt:tt )* ) => {
		{
			Diagnostic::spanned($span, Level::Error, format!( $( $tt )* )).emit();
			::std::process::exit(0)
		}
	};
}

#[derive(Debug)]
enum Node {
	Text(String),
	TextBlock(TokenStream),
	TextBlockIter(TokenStream),
	HtmlBlock(TokenStream),
	HtmlBlockIter(TokenStream),
	Element(Element)
}

#[derive(Debug)]
struct Element {
	name:  Ident,
	attrs: Vec<Attribute>,
	nodes: Vec<Node>
}

#[derive(Debug)]
struct Attribute {
	key: Ident,
	val: AttributeValue
}

#[derive(Debug)]
enum AttributeValue {
	Text(String),
	Block(TokenStream)
}

#[proc_macro]
pub fn html(item: TokenStream) -> TokenStream {
	let tokens = proc_macro2::TokenStream::from(item);
	let tokens: TokenStream = quote! {
		<DocumentFragment>
			#tokens
		</DocumentFragment>
	}.into();

	let tokens = generate(parse(&mut tokens.into_iter().collect()));
	TokenStream::from(quote! { (|| #tokens)() })
}

fn parse(tokens: &mut VecDeque<TokenTree>) -> Element {
	let mut attrs = Vec::new();
	let mut text  = String::new();
	let mut nodes = Vec::new();

	match tokens.pop_front() {
		Some(TokenTree::Punct(v)) if v.as_char() == '<' => (),
		Some(t) => emit!(t.span(), "unexpected token `{}`, expected `<`", t),
		None    => emit!(Span::call_site(), "unexpected EOF, expected `<`")
	}

	let name = match tokens.pop_front() {
		Some(TokenTree::Ident(v)) => v,
		Some(t) => emit!(t.span(), "unexpected token `{}`, expected <ident>", t),
		None    => emit!(Span::call_site(), "unexpected EOF, expected <ident>")
	};

	loop {
		match tokens.pop_front() {
			Some(TokenTree::Punct(v)) if v.as_char() == '>' => break,
			Some(TokenTree::Punct(v)) if v.as_char() == '/' => {
				match tokens.pop_front() {
					Some(TokenTree::Punct(v)) if v.as_char() == '>' => (),
					Some(t) => emit!(t.span(), "unexpected token `{}`, expected `>`", t),
					None    => emit!(Span::call_site(), "unexpected EOF, expected `>`")
				}

				return Element { name, attrs, nodes };
			}
			Some(TokenTree::Ident(key)) => {
				match tokens.pop_front() {
					Some(TokenTree::Punct(v)) if v.as_char() == '=' => (),
					Some(t) => emit!(t.span(), "unexpected token `{}`, expected `=`", t),
					None    => emit!(Span::call_site(), "unexpected EOF, expected `=`")
				}

				let val = match tokens.pop_front() {
					Some(TokenTree::Group(v)) if v.delimiter() == Delimiter::Brace => AttributeValue::Block(v.stream()),
					Some(TokenTree::Literal(v)) => {
						let v = v.to_string();
						let v = v.strip_prefix('"').unwrap_or(&v);
						let v = v.strip_suffix('"').unwrap_or(&v);
						AttributeValue::Text(v.to_string())
					},
					Some(t) => emit!(t.span(), "unexpected token `{}`, expected `{{ ... }}` or <literal>", t),
					None    => emit!(Span::call_site(), "unexpected EOF, expected {{ ... }} or <literal>")
				};

				attrs.push(Attribute { key, val });
			}
			Some(t) => emit!(t.span(), "unexpected token `{}`, expected <ident>, `/` or `>`", t),
			None    => emit!(Span::call_site(), "unexpected EOF, expected <ident> or `>`")
		}
	}

	loop {
		match tokens.pop_front() {
			Some(TokenTree::Group(v)) if v.delimiter() == Delimiter::Brace => {
				if !text.is_empty() {
					nodes.push(Node::Text(std::mem::take(&mut text)));
				}

				nodes.push(match tokens.front() {
					Some(TokenTree::Punct(p)) if p.as_char() == '*' => {
						tokens.pop_front();
						Node::TextBlockIter(v.stream())
					}
					Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
						tokens.pop_front();
						match tokens.front() {
							Some(TokenTree::Punct(p)) if p.as_char() == '*' => {
								tokens.pop_front();
								Node::HtmlBlockIter(v.stream())
							}
							_ => Node::HtmlBlock(v.stream())
						}
					}
					_ => Node::TextBlock(v.stream())
				})
			}
			Some(TokenTree::Punct(v)) if v.as_char() == '<' => {
				if !text.is_empty() {
					nodes.push(Node::Text(std::mem::take(&mut text)));
				}

				match tokens.pop_front() {
					Some(TokenTree::Punct(v)) if v.as_char() == '/' => break,
					Some(TokenTree::Ident(i)) => {
						tokens.push_front(TokenTree::Ident(i));
						tokens.push_front(TokenTree::Punct(v));
						nodes.push(Node::Element(parse(tokens)));
					}
					Some(t) => emit!(t.span(), "unexpected token `{}`, expected <ident> or `/`", t),
					None    => emit!(Span::call_site(), "unexpected EOF, expected <ident> or `/`")
				}
			}
			Some(t) => write!(&mut text, "{}", t).unwrap(),
			None    => emit!(name.span(), "tag not closed")
		}
	}

	let name_closing = match tokens.pop_front() {
		Some(TokenTree::Ident(v)) => v,
		Some(t) => emit!(t.span(), "unexpected token: `{}`, expected <ident>", t),
		None    => emit!(Span::call_site(), "unexpected EOF, expected <ident>")
	};

	match tokens.pop_front() {
		Some(TokenTree::Punct(v)) if v.as_char() == '>' => (),
		Some(t) => emit!(t.span(), "unexpected token: `{}`, expected `>`", t),
		None    => emit!(Span::call_site(), "unexpected EOF, expected `>`")
	};

	if name_closing.to_string() != name.to_string() {
		emit!(name_closing.span(), "unexpected closing tag `{}`, expected `{}`", &name_closing, &name);
	}

	Element { name, attrs, nodes }
}

fn generate(element: Element) -> proc_macro2::TokenStream {
	let attrs = element.attrs.into_iter()
		.map(|attr| match attr {
			Attribute { key, val: AttributeValue::Block(block) } if key.to_string().starts_with("on") => {
				let block    = proc_macro2::TokenStream::from(block);
				let set_fn   = proc_macro2::Ident::new(&format!("set_{}", key), proc_macro2::Span::call_site());
				let event_ty = proc_macro2::Ident::new(get_event_type(&key.to_string()), proc_macro2::Span::call_site());

				quote! {
					let closure = ::wasm_bindgen::closure::Closure::wrap(Box::new(move |event: ::web_sys::#event_ty| { #block }) as Box<dyn FnMut(::web_sys::#event_ty)>);
					e.dyn_ref::<HtmlElement>()
						.unwrap()
						.#set_fn(Some(::wasm_bindgen::JsCast::unchecked_ref(closure.as_ref())));
					closure.forget();
				}
			}
			Attribute { key, val: AttributeValue::Text(val) } => {
				let key = key.to_string();
				quote! { e.set_attribute(#key, #val)?; }
			}
			Attribute { key, val: AttributeValue::Block(val) } => {
				let key = key.to_string();
				let val = proc_macro2::TokenStream::from(val);
				quote! { e.set_attribute(#key, &(#val).to_string())?; }
			}
		});

	let nodes = element.nodes.into_iter()
		.map(|node| match node {
			Node::Element(v) => {
				let e = generate(v);
				quote! { e.append_with_node_1(::wasm_bindgen::JsCast::dyn_ref(&(#e)).unwrap())?; }
			}
			Node::Text(v) => {
				quote! { e.append_with_str_1(#v)?; }
			}
			Node::TextBlock(v) => {
				let block = proc_macro2::TokenStream::from(v);
				quote! { e.append_with_str_1(&(#block).to_string())?; }
			}
			Node::TextBlockIter(v) => {
				let block = proc_macro2::TokenStream::from(v);
				quote! {
					for v in (#block).into_iter() {
						e.append_with_str_1(&v.to_string())?;
					}
				}
			}
			Node::HtmlBlock(v) => {
				let block = proc_macro2::TokenStream::from(v);
				quote! { e.append_with_node_1(::wasm_bindgen::JsCast::dyn_ref(&(#block)).unwrap())?; }
			}
			Node::HtmlBlockIter(v) => {
				let block = proc_macro2::TokenStream::from(v);
				quote! {
					for v in (#block).into_iter() {
						e.append_with_node_1(::wasm_bindgen::JsCast::dyn_ref(&v).unwrap())?;
					}
				}
			}
		});

	let tag = element.name.to_string();

	if tag == "DocumentFragment" {
		quote! {
			{
				let e = document().create_document_fragment();
				#( #attrs )*
				#( #nodes )*
				Ok(e)
			}
		}
	} else {
		quote! {
			{
				let e = document().create_element(#tag)?;
				#( #attrs )*
				#( #nodes )*
				e
			}
		}
	}
}

fn get_event_type(listener: &str) -> &'static str {
	match listener.strip_prefix("on").unwrap_or(listener) {
		"click" | "mouseup" | "mousedown"         => "MouseEvent",
		"touchstart" | "touchmove"                => "TouchEvent",
		"focus" | "blur" | "focusin" | "focusout" => "FocusEvent",
		"keydown" | "keypress" | "keyup"          => "KeyboardEvent",
		"wheel"                                   => "WheelEvent",
		"input"                                   => "InputEvent",
		_                                         => "Event"
	}
}