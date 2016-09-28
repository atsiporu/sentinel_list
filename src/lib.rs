#![feature(pub_restricted)]

pub mod sentinel_list
{
	use std::ptr;
	use std::fmt::Debug;
	
	pub struct Link<T>
	{
		pub(sentinel_list) next: *mut Link<T>,
		pub(sentinel_list) prev: *mut Link<T>,
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

	pub(super) fn new_sentinel<T>() -> Handle<T>
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

	pub(super) fn new_handle<T>(v: T) -> Handle<T>
	{
		Box::new(Link::new(v))
	}

	pub(super) fn insert_after<T>(after: &mut Handle<T>, h: &mut Handle<T>)
	{
		let n: *mut _ = after.next;

		h.prev = &mut **after;
		h.next = n;
		
		after.next = &mut **h;
		
		let n = unsafe { &mut *h.next };
		n.prev = &mut **h;
	}

	fn debug_print<T: Debug>(s: &mut Handle<T>)
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
	use sentinel_list;

	#[test]
	fn it_works() {
		let mut s = sentinel_list::new_sentinel();
	}
}
