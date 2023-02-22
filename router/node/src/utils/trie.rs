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

#![allow(dead_code)]

use {std::{sync::{Arc, Weak}, ops, iter::FromIterator}, smol::lock::RwLock};

pub struct TrieNode<T>(Arc<TrieNodeInner<T>>);

impl<T> Clone for TrieNode<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T> TrieNode<T> {
	pub fn new(parent: &Self, path: impl ToString, data: Option<T>) -> Self {
		Self(Arc::new(TrieNodeInner::new(Some(&parent.0), path, data)))
	}
	
	pub fn from_inner(inner: TrieNodeInner<T>) -> Self {
		Self(Arc::new(inner))
	}
	
	pub fn into_inner(self) -> Result<TrieNodeInner<T>, Self> {
		Arc::try_unwrap(self.0).map_err(Self)
	}
	
	pub fn parent(&self) -> Option<Self> {
		self.0.parent
			.as_ref()
			.and_then(|v| v.upgrade())
			.map(Self)
	}
	
	#[allow(clippy::needless_lifetimes)]
	pub async fn child<'a>(&self, mut path: &'a str) -> Result<Self, (Self, &'a str, usize)> {
		let mut node = self.clone();
		
		while !path.is_empty() {
			match node.0.child(path).await {
				Ok((n, p)) => { node = n; path = p; }
				Err(i) => return Err((node, path, i))
			}
		}
		
		Ok(node)
	}
	
	pub async fn get(&self, path: &str) -> Option<Self> {
		self.child(path).await.ok()
	}
	
	pub async fn insert(&self, path: &str, data: T) -> Option<T> {
		match self.child(path).await {
			Ok(node) => std::mem::replace(&mut*node.data.write().await, Some(data)),
			Err((node, path, i)) => {
				let mut children = node.children.write().await;
				
				// find the sibling node with a common prefix
				
				let sibling_node = if i > 0 && children[i - 1].path.as_bytes()[0] == path.as_bytes()[0] {
					Some(i - 1)
				} else if i < children.len() && children[i].path.as_bytes()[0] == path.as_bytes()[0] {
					Some(i)
				} else {
					None
				};
				
				// insert new node
				
				if let Some(i) = sibling_node {
					let eq_len = children[i].path
						.bytes()
						.zip(path.bytes())
						.take_while(|(ch0, ch1)| ch0 == ch1)
						.count();
					
					let parent       = Some(Arc::downgrade(&node.0));
					let path_eq      = path[..eq_len].to_string().into_boxed_str();
					let children_old = RwLock::new(children[i].children.read().await.clone()); // TODO set new parent
					let data_old     = RwLock::new(children[i].data.write().await.take());
					let data_new     = RwLock::new(Some(data));
					
					match (children[i].path.len() > eq_len, path.len() > eq_len) {
						(true, true) => {
							let prefix_node = TrieNode::from_inner(TrieNodeInner {
								parent,
								path:     path_eq,
								children: RwLock::new(Vec::with_capacity(2)),
								data:     RwLock::new(None)
							});
							
							{
								let mut lock = prefix_node.children.write().await;
								
								lock.push(TrieNode::from_inner(TrieNodeInner {
									parent:   Some(Arc::downgrade(&prefix_node.0)),
									path:     children[i].path[eq_len..].to_string().into_boxed_str(),
									children: children_old,
									data:     data_old
								}));
								
								lock.push(TrieNode::from_inner(TrieNodeInner {
									parent:   Some(Arc::downgrade(&prefix_node.0)),
									path:     path[eq_len..].to_string().into_boxed_str(),
									children: RwLock::new(Vec::new()),
									data:     data_new
								}));
								
								lock.sort_by_key(|v| v.path.to_string());
							}
							
							children[i] = prefix_node;
						}
						(true, false) => {
							let prefix_node = TrieNode::from_inner(TrieNodeInner {
								parent,
								path:     path_eq,
								children: RwLock::new(Vec::with_capacity(1)),
								data:     data_new
							});
							
							let suffix_node = TrieNode::from_inner(TrieNodeInner {
								parent:   Some(Arc::downgrade(&prefix_node.0)),
								path:     children[i].path[eq_len..].to_string().into_boxed_str(),
								children: children_old,
								data:     data_old
							});
							
							prefix_node.children.write().await.push(suffix_node);
							children[i] = prefix_node;
						}
						(false, true) => {
							let prefix_node = TrieNode::from_inner(TrieNodeInner {
								parent,
								path:     path_eq,
								children: children_old,
								data:     data_old
							});
							
							let suffix_node = TrieNode::from_inner(TrieNodeInner {
								parent:   Some(Arc::downgrade(&prefix_node.0)),
								path:     path[eq_len..].to_string().into_boxed_str(),
								children: RwLock::new(Vec::new()),
								data:     data_new
							});
							
							{
								let mut children = prefix_node.children.write().await;
								let i = children.binary_search_by_key(&&prefix_node.path, |v| &v.path).unwrap_err();
								children.insert(i, suffix_node);
							}
							
							children[i] = prefix_node;
						}
						(false, false) => unreachable!()
					}
				} else {
					children.insert(i, Self::new(&node, path, Some(data)));
				}
				
				None
			}
		}
	}
	
	pub async fn remove(&self, path: &str) -> Option<T> {
		match self.child(path).await {
			Ok(node) => node.0.remove().await,
			Err(_) => None
		}
	}
	
	pub async fn delete(&self, path: &str) -> bool {
		match self.child(path).await {
			Ok(node) => node.0.delete().await,
			Err(_) => false
		}
	}
}

impl<'a, T, S: AsRef<str>> FromIterator<(S, T)> for TrieNode<T> {
	fn from_iter<I: IntoIterator<Item = (S, T)>>(iter: I) -> Self {
		let root = Self::default();
		
		for (path, data) in iter {
			smol::block_on(root.insert(path.as_ref(), data));
		}
		
		root
	}
}

impl<T> Default for TrieNode<T> {
	fn default() -> Self {
		Self(Arc::new(TrieNodeInner::new(None, "", None)))
	}
}

impl<T> ops::Deref for TrieNode<T> {
	type Target = TrieNodeInner<T>;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub struct TrieNodeInner<T> {
	pub parent:   Option<Weak<TrieNodeInner<T>>>,
	pub path:     Box<str>,
	pub children: RwLock<Vec<TrieNode<T>>>,
	pub data:     RwLock<Option<T>>
}

impl<T> TrieNodeInner<T> {
	pub fn new(parent: Option<&Arc<Self>>, path: impl ToString, data: Option<T>) -> Self {
		Self {
			parent:   parent.map(Arc::downgrade),
			path:     path.to_string().into_boxed_str(),
			children: RwLock::new(Vec::new()),
			data:     RwLock::new(data)
		}
	}
	
	#[allow(clippy::needless_lifetimes)]
	pub async fn child<'a>(&self, path: &'a str) -> Result<(TrieNode<T>, &'a str), usize> {
		let children = self.children.read().await;
		children.binary_search_by(|v| (&*v.path).cmp(&path[..v.path.len().min(path.len())]))
			.map(|i| (children[i].clone(), path.strip_prefix(&*children[i].path).unwrap()))
	}
	
	pub async fn remove(&self) -> Option<T> {
		if let Some(parent) = self.parent.as_ref().and_then(Weak::upgrade) {
			let children = self.children.write().await;
			let mut parent_children = parent.children.write().await;
			
			if children.len() <= 1 {
				match (&**children, parent_children.iter()
					.enumerate()
					.find(|(_, n)| std::ptr::eq(&*n.0, self)))
				{
					([], Some((i, _)))      => { parent_children.remove(i); },
					([child], Some((i, _))) => parent_children[i] = TrieNode::from_inner(Self {
						parent:   Some(Arc::downgrade(&parent)),
						path:     (self.path.to_string() + &*child.path).into_boxed_str(),
						children: RwLock::new(std::mem::take(&mut*child.children.write().await)),
						data:     RwLock::new(child.data.write().await.take())
					}),
					_ => ()
				}
			}
		}
		
		self.data.write().await.take()
	}
	
	pub async fn delete(&self) -> bool {
		if let Some(parent) = self.parent.as_ref().and_then(Weak::upgrade) {
			let mut children = parent.children.write().await;
			match children.iter()
				.enumerate()
				.find(|(_, n)| std::ptr::eq(&*n.0, self))
			{
				Some((idx, _)) => children.remove(idx),
				None => return false
			};
			
			true
		} else {
			false
		}
	}
}

impl<T: std::fmt::Debug> std::fmt::Debug for TrieNode<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(std::any::type_name::<Self>())
			.field("path", &self.path)
			.field("data", &self.data)
			.field("children", &self.children)
			.finish()
	}
}