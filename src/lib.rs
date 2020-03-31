use failure::err_msg;
use libc::{c_char, c_int, c_uint, size_t};
use mikack::{
    extractors,
    models::{self, FromLink, FromUrl},
};
use std::{
    ffi::{CStr, CString},
    mem, ptr, slice,
};

mod error_handling;
mod structs;
pub use error_handling::*;
use structs::*;

/// 获取全部平台列表
/// 不发生错误
#[no_mangle]
pub extern "C" fn platforms() -> *mut CArray<Platform> {
    let platforms = CArray::from(extractors::platforms());

    Box::into_raw(Box::new(platforms))
}

/// 释放平台列表内存
/// 不发生错误
#[no_mangle]
pub extern "C" fn free_platform_array(ptr: *mut CArray<Platform>) {
    unsafe {
        let array = Box::from_raw(ptr);
        if array.len == 0 {
            return;
        }
        slice::from_raw_parts_mut(array.data, array.len)
            .iter_mut()
            .map(|p: &mut Platform| {
                CString::from_raw(p.domain);
                CString::from_raw(p.name);
                CString::from_raw(p.favicon);
                free_tag_array(p.tags);
            })
            .for_each(drop);
        mem::drop(Box::from_raw(array.data));
    }
}

/// 获取全部标签列表
/// 不发生错误
#[no_mangle]
pub extern "C" fn tags() -> *mut CArray<Tag> {
    create_tags_ptr(&models::Tag::all())
}

/// 释放标签列表内存
/// 不发生错误
#[no_mangle]
pub extern "C" fn free_tag_array(ptr: *mut CArray<Tag>) {
    unsafe {
        let array = Box::from_raw(ptr);
        if array.len == 0 {
            return;
        }
        slice::from_raw_parts_mut(array.data, array.len)
            .iter_mut()
            .map(|t: &mut Tag| {
                CString::from_raw(t.name);
            })
            .for_each(drop);
        mem::drop(Box::from_raw(array.data));
    }
}

/// 从枚举值列表转换为标签模型列表
pub fn enumed_tags(values: &[i32]) -> Vec<models::Tag> {
    values
        .iter()
        .map(|v| models::Tag::from_i32(*v))
        .filter(|r| r.is_some())
        .map(|r| r.unwrap())
        .collect::<Vec<_>>()
}

/// 查找平台列表
/// includes:       包含标签枚举值数组
/// includes_len:   包含标签列表的个数
/// excludes:       排除标签枚举值数组
/// excludes_len:   排除标签列表的个数
/// 不发生错误
#[no_mangle]
pub unsafe extern "C" fn find_platforms(
    includes: *mut c_int,
    includes_len: size_t,
    excludes: *mut c_int,
    excludes_len: size_t,
) -> *mut CArray<Platform> {
    let includes = enumed_tags(slice::from_raw_parts_mut(includes, includes_len));
    let excludes = enumed_tags(slice::from_raw_parts_mut(excludes, excludes_len));

    let platforms = CArray::from(&extractors::find_platforms(includes, excludes));

    Box::into_raw(Box::new(platforms))
}

type ExtractorPtr = *const dyn extractors::Extractor;

/// 获取 Extractor
/// domian: 域名字符串
/// 不发生错误
#[no_mangle]
pub unsafe extern "C" fn get_extr(domian: *mut c_char) -> *mut ExtractorPtr {
    let domain = CStr::from_ptr(domian).to_str().unwrap();

    Box::into_raw(Box::new(
        extractors::get_extr(domain).unwrap().as_ref() as ExtractorPtr
    ))
}

/// 获取漫画列表
/// extr_ptr:  Extractor 的指针
/// *更新错误并返回空指针*
#[no_mangle]
pub extern "C" fn index(extr_ptr: *mut ExtractorPtr, page: c_uint) -> *const CArray<Comic> {
    let extr_ptr = unsafe { Box::from_raw(extr_ptr) };
    let extr = unsafe { &**extr_ptr };

    match extr.index(page) {
        Ok(comics) => {
            let array = CArray::from(comics);
            Box::into_raw(Box::new(array))
        }
        Err(e) => {
            update_last_error(e);
            ptr::null()
        }
    }
}

/// 释放漫画列表内存
#[no_mangle]
pub unsafe extern "C" fn free_comic_array(ptr: *mut CArray<Comic>) {
    if ptr.is_null() {
        return;
    }
    let array = Box::from_raw(ptr);
    if array.len == 0 {
        return;
    }
    slice::from_raw_parts_mut(array.data, array.len)
        .iter_mut()
        .map(|c: &mut Comic| {
            CString::from_raw(c.title);
            CString::from_raw(c.url);
            CString::from_raw(c.cover);
            free_chapter_array(c.chapters);
        })
        .for_each(drop);
    mem::drop(Box::from_raw(array.data));
}

/// 释放漫画内存
#[no_mangle]
pub unsafe extern "C" fn free_comic(ptr: *mut Comic) {
    let comic = Box::from_raw(ptr);
    CString::from_raw(comic.title);
    CString::from_raw(comic.url);
    CString::from_raw(comic.cover);
    free_chapter_array(comic.chapters);
}

/// 搜索漫画
/// extr_ptr: Extractor 的指针
/// keywords: 关键字字符串
/// *更新错误并返回空指针*
#[no_mangle]
pub extern "C" fn search(
    extr_ptr: *mut ExtractorPtr,
    keywords: *const c_char,
) -> *const CArray<Comic> {
    let extr_ptr = unsafe { Box::from_raw(extr_ptr) };
    let extr = unsafe { &**extr_ptr };
    let keywords = unsafe { CStr::from_ptr(keywords).to_str().unwrap() };

    match extr.search(keywords) {
        Ok(comics) => {
            let array = CArray::from(comics);
            Box::into_raw(Box::new(array))
        }
        Err(e) => {
            update_last_error(e);
            ptr::null()
        }
    }
}

/// 加载章节列表，返回填充数据后的漫画结构
/// extr_ptr: Extractor 的指针
/// ext_url: 外部 URL 字符串
/// ext_title: 外部标题字符串
/// *更新错误并返回空指针*
/// TODO: 未来此函数会将漫画指针作为唯一参数，无返回值
#[no_mangle]
pub extern "C" fn chapters(
    extr_ptr: *mut ExtractorPtr,
    ext_url: *const c_char,
    ext_title: *const c_char,
) -> *const Comic {
    let extr_ptr = unsafe { Box::from_raw(extr_ptr) };
    let extr = unsafe { &**extr_ptr };
    let url = unsafe { CStr::from_ptr(ext_url).to_str().unwrap() };
    let title = unsafe { CStr::from_ptr(ext_title).to_str().unwrap() };

    let comic = &mut models::Comic::from_link(title, url);
    match extr.fetch_chapters(comic) {
        Ok(()) => {
            let url_ptr = CString::new(url.as_bytes()).unwrap().into_raw();
            let title_ptr = CString::new(title.as_bytes()).unwrap().into_raw();
            let cover_ptr = CString::new(comic.cover.as_bytes()).unwrap().into_raw();

            let chapters = CArray::from(&comic.chapters);
            let chapters_ptr = Box::into_raw(Box::new(chapters));

            Box::into_raw(Box::new(Comic {
                title: title_ptr,
                url: url_ptr,
                cover: cover_ptr,
                chapters: chapters_ptr,
            }))
        }
        Err(e) => {
            update_last_error(e);
            ptr::null()
        }
    }
}

/// 释放章节列表内存
#[no_mangle]
pub extern "C" fn free_chapter_array(ptr: *mut CArray<Chapter>) {
    unsafe {
        let array = Box::from_raw(ptr);
        if array.len == 0 {
            return;
        }
        slice::from_raw_parts_mut(array.data, array.len)
            .iter_mut()
            .map(|c: &mut Chapter| {
                CString::from_raw(c.title);
                CString::from_raw(c.url);
                free_headers(c.page_headers);
            })
            .for_each(drop);
        mem::drop(Box::from_raw(array.data));
    }
}

/// 释放 Headers 内存
pub unsafe fn free_headers(ptr: *mut CArray<KV<*mut c_char, *mut c_char>>) {
    let array = Box::from_raw(ptr);
    if array.len == 0 {
        return;
    }
    slice::from_raw_parts_mut(array.data, array.len)
        .iter_mut()
        .map(|kv| {
            CString::from_raw(kv.key);
            CString::from_raw(kv.value);
        })
        .for_each(drop);
    mem::drop(Box::from_raw(array.data));
}

/// 创建章节指针
/// ext_url: 外部 URL 字符串
/// TODO: 未来此函数将删除，改为调用方自行创建章节指针
#[no_mangle]
pub extern "C" fn create_chapter_ptr(ext_url: *const c_char) -> *mut models::Chapter {
    let url = unsafe { CStr::from_ptr(ext_url).to_str().unwrap() };

    Box::into_raw(Box::new(models::Chapter::from_url(url)))
}

/// 创建页面迭代器（需要外部章节指针）
/// extr_ptr: Extractor 的指针
/// chapter_ptr: 章节的指针（由 `create_chapter_ptr` 函数提供）
/// *更新错误并返回空指针*
#[no_mangle]
pub extern "C" fn create_page_iter<'a>(
    extr_ptr: *mut ExtractorPtr,
    chapter_ptr: *mut models::Chapter,
) -> *const CreatedPageIter<'a> {
    let ptr = unsafe { Box::from_raw(extr_ptr) };
    let extr = unsafe { &**ptr };
    let chapter = unsafe { &mut *chapter_ptr };

    match extr.pages_iter(chapter) {
        Ok(iter) => {
            let mut iter = Box::new(iter);
            let title = iter.chapter_title_clone();
            let count = iter.total;
            let headers = &iter.chapter.page_headers;

            let ptr = Box::into_raw(Box::new(CreatedPageIter {
                count,
                title: CString::new(title.as_bytes()).unwrap().into_raw(),
                headers: create_headers_ptr(headers),
                iter: &mut *iter,
            }));
            mem::forget(iter);

            ptr
        }
        Err(e) => {
            update_last_error(e);
            ptr::null()
        }
    }
}

/// 释放已创建的页面迭代器
#[no_mangle]
pub extern "C" fn free_created_page_iter(ptr: *mut CreatedPageIter) {
    unsafe {
        let created_page_iter = Box::from_raw(ptr);
        CString::from_raw(created_page_iter.title);
        free_headers(created_page_iter.headers);
        Box::from_raw(created_page_iter.iter);
    };
}

/// 翻页（仅返回地址）
/// iter: 页面迭代器
/// *更新并返回错误*
/// TODO: 未来此函数将返回页面结构指针
#[no_mangle]
pub extern "C" fn next_page<'a>(iter: *mut extractors::ChapterPages<'a>) -> *const c_char {
    let iter = unsafe { &mut *iter };
    if let Some(page_r) = iter.next() {
        match page_r {
            Ok(page) => CString::new(page.address.as_bytes()).unwrap().into_raw(),
            Err(e) => {
                update_last_error(e);
                ptr::null()
            }
        }
    } else {
        // 没有下一页了
        update_last_error(err_msg("No next page"));
        ptr::null()
    }
}

/// 释放字符串内存
#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    unsafe {
        CString::from(CStr::from_ptr(ptr));
    }
}

// Fix `cannot locate symbol : "__assert_fail"` for Android platform
#[no_mangle]
pub extern "C" fn __assert_fail() {}
