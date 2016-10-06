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
use std::ops::{Deref, DerefMut};


pub trait ListHandle<T>
{
	fn unlink(self) -> T;
	fn as_ref(&self) -> &T;
}

pub struct Iter<'a, T: 'a>
{
	next: &'a Link<T>,
}

pub struct IterMut<'a, T: 'a>
{
	next: Option<&'a mut Link<T>>,
}

pub struct List<T>
{
	sentinel: Handle<T>,
}

#[derive(PartialEq)]
struct Link<T>
{
	pub next: *mut Link<T>,
	pub prev: *mut Link<T>,
	pub value: Option<T>,
}

struct Handle<T>(*mut Link<T>);

impl<T> List<T>
{
	pub fn new() -> Self
	{
		List {
			sentinel: Handle::new_sentinel(),
		}
	}

	pub fn push_head(&mut self, e: T) -> impl ListHandle<T>
	{
		let mut h = Handle::new(e);
		insert_after(&mut self.sentinel, &mut h);
		h
	}

	pub fn push_tail(&mut self, e: T) -> impl ListHandle<T>
	{
		let mut h = Handle::new(e);
		insert_after(unsafe {&mut *self.sentinel.prev}, &mut h);
		h
	}

	pub fn peek_head(&self) -> Option<&T>
	{
		let link = unsafe { &*self.sentinel.next };
		link.value.as_ref()
	}
	
	pub fn peek_head_mut(&mut self) -> Option<&mut T>
	{
		let link = unsafe { &mut *self.sentinel.next };
		link.value.as_mut()
	}

	pub fn peek_tail(&self) -> Option<&T>
	{
		let link = unsafe { &*self.sentinel.prev };
		link.value.as_ref()
	}

	pub fn peek_tail_mut(&mut self) -> Option<&mut T>
	{
		let link = unsafe { &mut *self.sentinel.prev };
		link.value.as_mut()
	}

	pub fn iter(&self) -> Iter<T>
	{
		Iter { next: unsafe {&*self.sentinel.next} }
	}

	pub fn iter_mut(&mut self) -> IterMut<T>
	{
		let next = Some(unsafe {&mut *self.sentinel.next});
		let inext = next.map(|v| {
			v
		});
		IterMut { next: inext }
	}
}

impl<'a, T> Iterator for Iter<'a, T>
{
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item>
	{
		self.next.value.as_ref().and_then(|v| {
			self.next = unsafe {&*self.next.next};
			Some(v)
		})
	}
}

impl<'a, T> Iterator for IterMut<'a, T>
{
	type Item = &'a mut T;

	fn next(&mut self) -> Option<Self::Item>
	{
		self.next.take().and_then(|link| {
			self.next = Some(unsafe {&mut *link.next});
			link.value.as_mut().map(|v| {
				v
			})
		})
	}
}

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

    // just unlinks
    // after calling this it's unsafe to use
    // this Handle without adjusting prev, next
    fn unlink(&mut self)
    {
        let prev = unsafe { &mut *self.prev };
        let next = unsafe { &mut *self.next };
        next.prev = prev;
        prev.next = next;
    }

}
impl<T> Handle<T>
{
    fn new(v: T) -> Self
    {
	    Handle(Box::into_raw(Box::new(Link::new(v))))
    }

	fn new_sentinel() -> Self 
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
		Handle(Box::into_raw(h))
	}
    
    fn into_inner(self) -> Option<T>
    {
        let mut h = self;
        let link = unsafe { &mut *h.0 };
        link.unlink();
        link.value.take()
    }
}

impl<T: fmt::Debug> fmt::Debug for Link<T>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Link {{ prev: {:p}, next: {:p}, value: {:?} }}", self.prev, self.next, self.value)
	}
}

impl<T> Drop for Handle<T>
{
    fn drop(&mut self)
    {
        let link = unsafe { &mut *self.0 };
        link.unlink();
        if !self.0.is_null() {
            let h = unsafe { Box::from_raw(self.0) };
            drop(h);
        }
        // not sure if this matters
        //println!("Drop");
    }
}

impl<T> ListHandle<T> for Handle<T>
{
	fn unlink(self) -> T
	{
		// if you call it on the sentinel 
		// you deserve to get what's coming
		self.into_inner().unwrap()
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

impl<T> Deref for Handle<T>
{
	type Target = Link<T>;
	fn deref(&self) -> &Self::Target
	{
		unsafe {&*self.0}
	}
}

impl<T> DerefMut for Handle<T>
{
	fn deref_mut(&mut self) -> &mut Self::Target
	{
		unsafe {&mut *self.0}
	}
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

#[allow(dead_code)]
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
	let mut s = Handle::new_sentinel();
	let mut h1 = Handle::new(1);
	let mut h2 = Handle::new(2);
	let mut h3 = Handle::new(3);
	
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

	h2.into_inner();

	assert_eq!(unsafe {&*s.next}, &*h1);
	assert_eq!(unsafe {&*h1.prev}, &*s);
	
	assert_eq!(unsafe {&*h1.next}, &*h3);
	assert_eq!(unsafe {&*h3.prev}, &*h1);
	
	assert_eq!(unsafe {&*h3.next}, &*s);
	assert_eq!(unsafe {&*s.prev}, &*h3);
	
	let mut h2 = Handle::new(2);
	insert_after(&mut h1, &mut h2);
	
	h1.into_inner();
	assert_eq!(unsafe {&*s.next}, &*h2);
	h2.into_inner();
	assert_eq!(unsafe {&*s.next}, &*h3);
	assert_eq!(unsafe {&*s.prev}, &*h3);
	assert_eq!(unsafe {&*h3.prev}, &*s);
	assert_eq!(unsafe {&*h3.next}, &*s);
	h3.into_inner();
	assert_eq!(unsafe {&*s.prev}, &*s);
	assert_eq!(unsafe {&*s.next}, &*s);
	
	s.into_inner();
}

#[cfg(test)]
#[test]
fn iter_test()
{
	let l = &mut List::new();
	let h1 = l.push_head(1);
    {
	let h2 = l.push_tail(2);
    }
    let h3 = l.push_tail(3);

	let mut i = l.iter();
	assert_eq!(Some(&1), i.next());
	//assert_eq!(Some(&2), i.next());
	assert_eq!(Some(&3), i.next());
	assert_eq!(None, i.next());

    h1.unlink();
    //h2.unlink();
    h3.unlink();
}

#[cfg(test)]
#[test]
fn iter_mut_test()
{
	let l = &mut List::new();
	let h1 = l.push_head(1);
	let h2 = l.push_tail(2);
	let h3 = l.push_tail(3);

	{
	let mut i = l.iter_mut();
	assert_eq!(Some(&mut 1), i.next());
	assert_eq!(Some(&mut 2), i.next());
	assert_eq!(Some(&mut 3), i.next());
	assert_eq!(None, i.next());
	}

	l.iter_mut().fold(3, |i, v| {
		*v = i;
		i - 1
	});

	{
	let mut i = l.iter_mut();
	assert_eq!(Some(&mut 3), i.next());
	assert_eq!(Some(&mut 2), i.next());
	assert_eq!(Some(&mut 1), i.next());
	assert_eq!(None, i.next());
	}
	
	assert_eq!(&3, h1.as_ref());
	assert_eq!(&2, h2.as_ref());
	assert_eq!(&1, h3.as_ref());
}

#[cfg(test)]
//#[test]
fn test_drop()
{
    let line = &mut String::new();
    
    std::io::stdin().read_line(line);
	
    let mut s = Handle::new_sentinel();
    let mut vec = vec![];

    for i in 1..100000 {
        let mut h = Handle::new(vec![0u32;1000]);
        insert_after(&mut s,  &mut h);
        vec.push(h);
    }
    
    std::io::stdin().read_line(line);

    while !vec.is_empty() {
        vec.pop().unwrap().into_inner();
    }
    
    std::io::stdin().read_line(line);
}
