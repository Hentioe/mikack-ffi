use libc::{c_char, c_int, c_uint, size_t};
use mikack::{extractors, models};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    mem, slice,
};

#[derive(Debug)]
#[repr(C)]
pub struct Platform {
    pub domain: *mut c_char,
    pub name: *mut c_char,
}

#[derive(Debug)]
#[repr(C)]
pub struct CArray<T> {
    pub len: size_t,
    pub data: *mut T,
}

impl From<&HashMap<String, String>> for CArray<Platform> {
    fn from(map: &HashMap<String, String>) -> Self {
        let mut data_items = vec![];
        for (domain, name) in map.iter() {
            data_items.push(Platform {
                domain: CString::new(domain.as_bytes()).unwrap().into_raw(),
                name: CString::new(name.as_bytes()).unwrap().into_raw(),
            });
        }

        let len = data_items.len();
        let mut boxed_data = data_items.into_boxed_slice();
        let data = boxed_data.as_mut_ptr();
        mem::forget(boxed_data);

        CArray { len, data }
    }
}

#[no_mangle]
pub extern "C" fn platforms() -> *mut CArray<Platform> {
    let platforms = CArray::from(extractors::platforms());

    Box::into_raw(Box::new(platforms))
}

#[no_mangle]
pub extern "C" fn free_platform_array(ptr: *mut CArray<Platform>) {
    unsafe {
        let array = Box::from_raw(ptr);
        slice::from_raw_parts_mut(array.data, array.len)
            .iter_mut()
            .map(|p: &mut Platform| {
                CString::from_raw(p.domain);
                CString::from_raw(p.name);
            })
            .for_each(drop);
        mem::drop(Box::from_raw(array.data));
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Tag {
    value: i32,
    name: *mut c_char,
}

impl From<&models::Tag> for Tag {
    fn from(t: &models::Tag) -> Self {
        Tag {
            value: *t as i32,
            name: CString::new(t.to_string().as_bytes()).unwrap().into_raw(),
        }
    }
}

#[no_mangle]
pub extern "C" fn tags() -> *mut CArray<Tag> {
    let items = models::Tag::all()
        .iter()
        .map(|t| Tag::from(t))
        .collect::<Vec<_>>();

    let len = items.len();
    let mut boxed_data = items.into_boxed_slice();
    let data = boxed_data.as_mut_ptr();
    mem::forget(boxed_data);

    Box::into_raw(Box::new(CArray { len, data }))
}

#[no_mangle]
pub extern "C" fn free_tag_array(ptr: *mut CArray<Tag>) {
    unsafe {
        let array = Box::from_raw(ptr);
        slice::from_raw_parts_mut(array.data, array.len)
            .iter_mut()
            .map(|t: &mut Tag| {
                CString::from_raw(t.name);
            })
            .for_each(drop);
        mem::drop(Box::from_raw(array.data));
    }
}

pub fn enumed_tags(values: &[i32]) -> Vec<models::Tag> {
    values
        .iter()
        .map(|v| models::Tag::from_i32(*v))
        .filter(|r| r.is_some())
        .map(|r| r.unwrap())
        .collect::<Vec<_>>()
}

#[no_mangle]
pub extern "C" fn find_platforms(
    inc_ptr: *mut c_int,
    inc_len: size_t,
    exc_ptr: *mut c_int,
    exc_len: size_t,
) -> *mut CArray<Platform> {
    let includes = unsafe {
        let values = slice::from_raw_parts_mut(inc_ptr, inc_len);
        enumed_tags(values)
    };
    let excludes = unsafe {
        let values = slice::from_raw_parts_mut(exc_ptr, exc_len);
        enumed_tags(values)
    };

    let platforms = CArray::from(&extractors::find_platforms(includes, excludes));

    Box::into_raw(Box::new(platforms))
}

type ExtrPtr = *const dyn extractors::Extractor;

#[no_mangle]
pub extern "C" fn get_extr(domain_ptr: *mut c_char) -> *mut ExtrPtr {
    let domain = unsafe {
        let cstr = CStr::from_ptr(domain_ptr);
        cstr.to_str().unwrap()
    };

    Box::into_raw(Box::new(
        extractors::get_extr(domain).unwrap().as_ref() as ExtrPtr
    ))
}

#[derive(Debug)]
#[repr(C)]
pub struct Comic {
    title: *mut c_char,
    url: *mut c_char,
    cover: *mut c_char,
}

impl From<&models::Comic> for Comic {
    fn from(c: &models::Comic) -> Self {
        Comic {
            title: CString::new(c.title.as_bytes()).unwrap().into_raw(),
            url: CString::new(c.url.as_bytes()).unwrap().into_raw(),
            cover: CString::new(c.cover.as_bytes()).unwrap().into_raw(),
        }
    }
}

impl From<Vec<models::Comic>> for CArray<Comic> {
    fn from(list: Vec<models::Comic>) -> Self {
        let len = list.len();
        let items = list.iter().map(|c| Comic::from(c)).collect::<Vec<_>>();

        let mut boxed_data = items.into_boxed_slice();
        let data = boxed_data.as_mut_ptr();
        mem::forget(boxed_data);

        CArray { len, data }
    }
}

#[no_mangle]
pub extern "C" fn index(extr_ptr_ptr: *mut ExtrPtr, page: c_uint) -> *mut CArray<Comic> {
    let ptr = unsafe { Box::from_raw(extr_ptr_ptr) };
    let extr = unsafe { &**ptr };

    let array = CArray::from(extr.index(page).unwrap());
    Box::into_raw(Box::new(array))
}

#[no_mangle]
pub extern "C" fn free_comic_array(ptr: *mut CArray<Comic>) {
    unsafe {
        let array = Box::from_raw(ptr);
        slice::from_raw_parts_mut(array.data, array.len)
            .iter_mut()
            .map(|c: &mut Comic| {
                CString::from_raw(c.title);
                CString::from_raw(c.url);
                CString::from_raw(c.cover);
            })
            .for_each(drop);
        mem::drop(Box::from_raw(array.data));
    }
}

#[no_mangle]
pub extern "C" fn search(
    extr_ptr_ptr: *mut ExtrPtr,
    keywords_ptr: *const c_char,
) -> *mut CArray<Comic> {
    let ptr = unsafe { Box::from_raw(extr_ptr_ptr) };
    let extr = unsafe { &**ptr };
    let keywords = unsafe { CStr::from_ptr(keywords_ptr).to_str().unwrap() };

    let array = CArray::from(extr.search(keywords).unwrap());
    Box::into_raw(Box::new(array))
}
