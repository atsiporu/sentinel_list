//! The `sentinel_list` crate provides a simple sentinel based list implementation.
//! Once you insert element to the list you are given back a handle into this list.
//! This handle allows you to remove element from the list in O(1) time.
//!
//! # Examples
//!
//! ```rust
//! extern crate sentinel_list;
//! use sentinel_list::*;
//!
//! fn main() {
//!	 let l = &mut List::new();
//!	 let h1 = l.push_head(1);
//!	 let h2 = l.push_tail(2);
//!	 let h3 = l.push_tail(3);
//!	 {
//!		assert_eq!(&1, (&h1).as_ref());
//!		assert_eq!(&1, l.peek_head().unwrap());
//!		assert_eq!(&3, l.peek_tail().unwrap());
//!	 }
//!	 let e1 = h1.unlink();
//!	 assert_eq!(&2, l.peek_head().unwrap());
//!	 let e3 = h3.unlink();
//!	 assert_eq!(&2, l.peek_tail().unwrap());
//!	 let e2 = h2.unlink();
//!	 assert_eq!(None, l.peek_tail());
//!	 assert_eq!(None, l.peek_head());
//!	 assert_eq!(1, e1);
//!	 assert_eq!(2, e2);
//!	 assert_eq!(3, e3);
//! }
//! ```

#![feature(conservative_impl_trait)]

use std::ptr;
use std::fmt;
use std::ops::Deref;


pub trait ListHandle<T>
{
	fn unlink(self) -> T;
	fn as_ref(&self) -> &T;
}

pub struct List<T>
{
	sentinel: Handle<T>,
}

impl<T> List<T>
{
	pub fn new() -> Self
	{
		List {
			sentinel: new_sentinel(),
		}
	}

	pub fn push_head(&mut self, e: T) -> impl ListHandle<T>
	{
		let mut h = new_handle(e);
		insert_after(&mut self.sentinel, &mut h);
		h
	}

	pub fn push_tail(&mut self, e: T) -> impl ListHandle<T>
	{
		let mut h = new_handle(e);
		insert_after(unsafe {&mut *self.sentinel.prev}, &mut h);
		h
	}

	pub fn peek_head(&self) -> Option<&T>
	{
		let link = unsafe { &*self.sentinel.next };
		link.value.as_ref()
	}
	
	pub fn peek_tail(&self) -> Option<&T>
	{
		let link = unsafe { &*self.sentinel.prev };
		link.value.as_ref()
	}
}

#[derive(PartialEq)]
struct Link<T>
{
	pub next: *mut Link<T>,
	pub prev: *mut Link<T>,
	pub value: Option<T>,
}

type Handle<T> = Box<Link<T>>;

impl<T> Link<T>
{
	fn new(v: T) -> Link<T>
	{
		Link {
			prev: ptr::null_mut(),
			next: ptr::null_mut(),
			value: Some(v),
		}
	}
}

impl<T: fmt::Debug> fmt::Debug for Link<T>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Link {{ prev: {:p}, next: {:p}, value: {:?} }}", self.prev, self.next, self.value)
	}
}

impl<T> ListHandle<T> for Handle<T>
{
	fn unlink(self) -> T
	{
		// if you call it on the sentinel 
		// you deserve to get what's coming
		unlink(self).unwrap()
	}

	fn as_ref(&self) -> &T
	{
		&self
	}
}

impl<T> Deref for Link<T>
{
	type Target = T;
	fn deref(&self) -> &Self::Target
	{
		// if you call it on the sentinel 
		// you deserve to get what's coming
		self.value.as_ref().unwrap()
	}
}

fn new_sentinel<T>() -> Handle<T>
{
	let mut h = Box::new(
		Link { 
			prev: ptr::null_mut(), 
			next: ptr::null_mut(),
			value: None,
		}
	);
	h.prev = &mut *h;
	h.next = &mut *h;
	h
}

fn new_handle<T>(v: T) -> Handle<T>
{
	Box::new(Link::new(v))
}

fn insert_after<T>(after: &mut Link<T>, h: &mut Link<T>)
{
	let n: *mut _ = after.next;

	h.prev = &mut *after;
	h.next = n;
	
	after.next = &mut *h;
	
	let n = unsafe { &mut *h.next };
	n.prev = &mut *h;
}

fn unlink<T>(h: Handle<T>) -> Option<T>
{
	let prev = unsafe { &mut *h.prev };
	let next = unsafe { &mut *h.next };
	next.prev = prev;
	prev.next = next;
    h.value
}

#[cfg(test)]
fn debug_print<T: fmt::Debug>(s: &mut Handle<T>)
{
	let mut h = s.next;
	while h != &mut **s {
		let n = unsafe { &*h };
		println!("{:?}", n);
		h = n.next;
	}
}

#[cfg(test)]
#[test]
fn link_unlink()
{
	let mut s = new_sentinel();
	let mut h1 = new_handle(1);
	let mut h2 = new_handle(2);
	let mut h3 = new_handle(3);
	
	insert_after(&mut s,  &mut h1);
	insert_after(&mut h1, &mut h2);
	insert_after(&mut h2, &mut h3);

	assert_eq!(unsafe {&*s.next}, &*h1);
	assert_eq!(unsafe {&*h1.prev}, &*s);

	assert_eq!(unsafe {&*h1.next}, &*h2);
	assert_eq!(unsafe {&*h2.prev}, &*h1);
	
	assert_eq!(unsafe {&*h2.next}, &*h3);
	assert_eq!(unsafe {&*h3.prev}, &*h2);
	
	assert_eq!(unsafe {&*h3.next}, &*s);
	assert_eq!(unsafe {&*s.prev}, &*h3);

	unlink(h2);

	assert_eq!(unsafe {&*s.next}, &*h1);
	assert_eq!(unsafe {&*h1.prev}, &*s);
	
	assert_eq!(unsafe {&*h1.next}, &*h3);
	assert_eq!(unsafe {&*h3.prev}, &*h1);
	
	assert_eq!(unsafe {&*h3.next}, &*s);
	assert_eq!(unsafe {&*s.prev}, &*h3);
	
	let mut h2 = new_handle(2);
	insert_after(&mut h1, &mut h2);
	
	unlink(h1);
	assert_eq!(unsafe {&*s.next}, &*h2);
	unlink(h2);
	assert_eq!(unsafe {&*s.next}, &*h3);
	assert_eq!(unsafe {&*s.prev}, &*h3);
	assert_eq!(unsafe {&*h3.prev}, &*s);
	assert_eq!(unsafe {&*h3.next}, &*s);
	unlink(h3);
	assert_eq!(unsafe {&*s.prev}, &*s);
	assert_eq!(unsafe {&*s.next}, &*s);
	
	unlink(s);
}
