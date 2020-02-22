use libc::size_t;
use mikack::extractors;
use std::{collections::HashMap, ffi::CString};

#[derive(Debug)]
#[repr(C)]
pub struct Platform {
    pub domain: CString,
    pub name: CString,
}

#[derive(Debug)]
#[repr(C)]
pub struct CArray<T> {
    pub len: size_t,
    pub data: *const T,
}

impl From<&HashMap<String, String>> for CArray<Platform> {
    fn from(map: &HashMap<String, String>) -> Self {
        let mut platforms = vec![];
        for (_i, (domain, name)) in map.iter().enumerate() {
            platforms.push(Platform {
                domain: CString::new(domain.as_bytes()).unwrap(),
                name: CString::new(name.as_bytes()).unwrap(),
            });
        }

        let len = platforms.len();
        let mut data = platforms.into_boxed_slice();
        let data_ptr = data.as_mut_ptr();
        std::mem::forget(data);

        CArray {
            len,
            data: data_ptr,
        }
    }
}

#[no_mangle]
pub extern "C" fn platforms() -> *mut CArray<Platform> {
    let platforms = CArray::from(extractors::platforms());

    Box::into_raw(Box::new(platforms))
}
