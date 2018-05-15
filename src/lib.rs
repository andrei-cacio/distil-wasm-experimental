extern crate image;

use image::{guess_format, ImageFormat};
use std::mem;
use std::slice;
use std::os::raw::c_void;

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
	let mut buf = Vec::with_capacity(size);
	let ptr = buf.as_mut_ptr();
	mem::forget(buf);

	return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn read_img(buff_ptr: *mut u8, buff_len: usize) -> i32 {
	let mut img: Vec<u8> = unsafe { Vec::from_raw_parts(buff_ptr, buff_len, buff_len) };

    if let Ok(format) = guess_format(&img) {
    	return match format {
    		ImageFormat::JPEG => 1,
    		ImageFormat::PNG => 2,
    		_ => 0,
    	};
    }

    return img.len() as i32;
}

fn main() {
	println!("Hello from rust 2");
}