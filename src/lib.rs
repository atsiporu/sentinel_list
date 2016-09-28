//! The `sentinel_list` crate provides a simple sentinel based list implementation.
//! Once you insert element to the list you are given back a handle into this list.
//! This handle allows you to remove element from the list in O(1) time.
//!
//! # Examples
//!
//! ```rust
//! extern crate sentinel_list;
//! use sentinel_list::{new_sentinel, new_handle, insert_after, debug_print};
//!
//! fn main() {
//!     let mut s = new_sentinel();
//!     let mut h1 = new_handle(1);
//!     let mut h2 = new_handle(2);
//!     let mut h3 = new_handle(3);
//!     let mut h4 = new_handle(4);
//! 
//!     println!("\nObjects:");
//!     println!("{:p} {:?}", s, s.value);
//!     println!("{:p} {:?}", h1, h1.value);
//!     println!("{:p} {:?}", h2, h2.value);
//!     println!("{:p} {:?}", h3, h3.value);
//!     println!("{:p} {:?}", h4, h4.value);
//!     
//!     insert_after(&mut s, &mut h1);
//!     insert_after(&mut h1, &mut h2);
//!     insert_after(&mut h2, &mut h3);
//!     insert_after(&mut h3, &mut h4);
//!     
//!     println!("\nLinked:");
//!     println!("{:p} {:?}", s, s.value);
//!     println!("{:p} {:?}", h1, h1.value);
//!     println!("{:p} {:?}", h2, h2.value);
//!     println!("{:p} {:?}", h3, h3.value);
//!     println!("{:p} {:?}", h4, h4.value);
//!     
//!     
//!     println!("\nDebug-print:");
//!     debug_print(&mut s);
//! }
//! ```

pub use self::sentinel_list::{new_handle, new_sentinel, Handle, insert_after, debug_print};

mod sentinel_list
{
	use std::ptr;
	use std::fmt::Debug;
	
	pub struct Link<T>
	{
		pub next: *mut Link<T>,
		pub prev: *mut Link<T>,
		pub value: Option<T>,
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
	}

	pub type Handle<T> = Box<Link<T>>;

	pub fn new_sentinel<T>() -> Handle<T>
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

	pub fn new_handle<T>(v: T) -> Handle<T>
	{
		Box::new(Link::new(v))
	}

	pub fn insert_after<T>(after: &mut Handle<T>, h: &mut Handle<T>)
	{
		let n: *mut _ = after.next;

		h.prev = &mut **after;
		h.next = n;
		
		after.next = &mut **h;
		
		let n = unsafe { &mut *h.next };
		n.prev = &mut **h;
	}

	pub fn debug_print<T: Debug>(s: &mut Handle<T>)
	{
		let mut h = s.next;
		while h != &mut **s {
			let n = unsafe { &*h };
			println!("{:?}", n.value);
			h = n.next;
		}
	}
}

#[cfg(test)]
mod tests {
	use sentinel_list::{new_sentinel, new_handle, debug_print, insert_after};

	#[test]
	fn it_works() {
        let mut s = new_sentinel();
        let mut h1 = new_handle(1);
        let mut h2 = new_handle(2);
        let mut h3 = new_handle(3);
        let mut h4 = new_handle(4);


        println!("\nObjects:");
        println!("{:p} {:?}", s, s.value);
        println!("{:p} {:?}", h1, h1.value);
        println!("{:p} {:?}", h2, h2.value);
        println!("{:p} {:?}", h3, h3.value);
        println!("{:p} {:?}", h4, h4.value);

        insert_after(&mut s, &mut h1);
        insert_after(&mut h1, &mut h2);
        insert_after(&mut h2, &mut h3);
        insert_after(&mut h3, &mut h4);
        
        println!("\nLinked:");
        println!("{:p} {:?}", s, s.value);
        println!("{:p} {:?}", h1, h1.value);
        println!("{:p} {:?}", h2, h2.value);
        println!("{:p} {:?}", h3, h3.value);
        println!("{:p} {:?}", h4, h4.value);


        println!("\nDebug-print:");
        debug_print(&mut s);
	}
}
