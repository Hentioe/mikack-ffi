use libc::{c_char, c_int, c_uint, size_t};
use mikack::{
    extractors,
    models::{self},
};
use std::{collections::HashMap, ffi::CString, mem};

#[derive(Debug)]
#[repr(C)]
pub struct Platform {
    pub domain: *mut c_char,
    pub name: *mut c_char,
    pub favicon: *mut c_char,
    pub is_usable: bool,
    pub is_searchable: bool,
    pub is_pageable: bool,
    pub is_https: bool,
    pub is_search_pageable: bool,
    pub tags: *mut CArray<Tag>,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tag {
    pub value: i32,
    pub name: *mut c_char,
}

#[derive(Debug)]
#[repr(C)]
pub struct CArray<T> {
    pub len: size_t,
    pub data: *mut T,
}

#[derive(Debug)]
#[repr(C)]
pub struct Comic {
    pub title: *mut c_char,
    pub url: *mut c_char,
    pub cover: *mut c_char,
    pub chapters: *mut CArray<Chapter>,
}

#[derive(Debug)]
#[repr(C)]
pub struct Chapter {
    pub title: *mut c_char,
    pub url: *mut c_char,
    pub which: c_uint,
    pub page_headers: *mut CArray<KV<*mut c_char, *mut c_char>>,
}

#[derive(Debug)]
#[repr(C)]
pub struct KV<K, V> {
    pub key: K,
    pub value: V,
}

#[derive(Debug)]
#[repr(C)]
pub struct CreatedPageIter<'a> {
    pub count: c_int,
    pub title: *mut c_char,
    pub headers: *mut CArray<KV<*mut c_char, *mut c_char>>,
    pub iter: *mut extractors::ChapterPages<'a>,
}

pub fn create_tags_ptr(tags: &Vec<models::Tag>) -> *mut CArray<Tag> {
    let items = tags.iter().map(|t| Tag::from(t)).collect::<Vec<_>>();

    let len = items.len();
    let mut boxed_data = items.into_boxed_slice();
    let data = boxed_data.as_mut_ptr();
    mem::forget(boxed_data);

    Box::into_raw(Box::new(CArray { len, data }))
}

pub fn create_headers_ptr(
    headers: &HashMap<String, String>,
) -> *mut CArray<KV<*mut c_char, *mut c_char>> {
    let headers_items = headers
        .iter()
        .map(|(header, value)| KV {
            key: CString::new(header.as_bytes()).unwrap().into_raw(),
            value: CString::new(value.as_bytes()).unwrap().into_raw(),
        })
        .collect::<Vec<_>>();
    let len = headers_items.len();
    let mut boxed_data = headers_items.into_boxed_slice();
    let data = boxed_data.as_mut_ptr();
    mem::forget(boxed_data);

    Box::into_raw(Box::new(CArray { len, data }))
}

impl From<&HashMap<String, String>> for CArray<Platform> {
    fn from(map: &HashMap<String, String>) -> Self {
        let mut data_items = vec![];
        for (domain, name) in map.iter() {
            let extr = extractors::get_extr(domain).unwrap();
            data_items.push(Platform {
                domain: CString::new(domain.as_bytes()).unwrap().into_raw(),
                name: CString::new(name.as_bytes()).unwrap().into_raw(),
                favicon: CString::new(extr.get_favicon().unwrap_or(&"").as_bytes())
                    .unwrap()
                    .into_raw(),
                is_usable: extr.is_usable(),
                is_searchable: extr.is_searchable(),
                is_pageable: extr.is_pageable(),
                is_https: extr.is_https(),
                is_search_pageable: extr.is_pageable_search(),
                tags: create_tags_ptr(extr.tags()),
            });
        }

        let len = data_items.len();
        let mut boxed_data = data_items.into_boxed_slice();
        let data = boxed_data.as_mut_ptr();
        mem::forget(boxed_data);

        CArray { len, data }
    }
}

impl From<&models::Tag> for Tag {
    fn from(t: &models::Tag) -> Self {
        Tag {
            value: *t as i32,
            name: CString::new(t.to_string().as_bytes()).unwrap().into_raw(),
        }
    }
}
impl From<&models::Chapter> for Chapter {
    fn from(c: &models::Chapter) -> Self {
        let page_headers = create_headers_ptr(&c.page_headers);
        Chapter {
            title: CString::new(c.title.as_bytes()).unwrap().into_raw(),
            url: CString::new(c.url.as_bytes()).unwrap().into_raw(),
            which: c.which,
            page_headers,
        }
    }
}

impl From<&Vec<models::Chapter>> for CArray<Chapter> {
    fn from(list: &Vec<models::Chapter>) -> Self {
        let len = list.len();
        let items = list.iter().map(|c| Chapter::from(c)).collect::<Vec<_>>();

        let mut boxed_data = items.into_boxed_slice();
        let data = boxed_data.as_mut_ptr();
        mem::forget(boxed_data);

        CArray { len, data }
    }
}

impl From<&models::Comic> for Comic {
    fn from(c: &models::Comic) -> Self {
        Comic {
            title: CString::new(c.title.as_bytes()).unwrap().into_raw(),
            url: CString::new(c.url.as_bytes()).unwrap().into_raw(),
            cover: CString::new(c.cover.as_bytes()).unwrap().into_raw(),
            chapters: Box::into_raw(Box::new(CArray::from(&c.chapters))),
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
