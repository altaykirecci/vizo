//! macOS integration module for handling document open events from Finder.
//!
//! When an application is launched via "Open With" or a file is double-clicked in Finder,
//! macOS delivers documents via low-level Apple Events (`kAEOpenDocuments` / `'odoc'`)
//! and AppKit delegate selectors (`application:openFile:`, `application:openFiles:`, `application:openURLs:`).
//!
//! Because default winit/eframe application delegates do not handle these events,
//! LaunchServices displays the error: "Vizo cannot open files in the Image Document format".
//!
//! This module installs both:
//! 1. A native Apple Event Manager handler via `AEInstallEventHandler` for `kAEOpenDocuments`.
//! 2. Dynamic Objective-C method handlers on `NSObject` for `application:openFile:` etc.
//!
//! When an open event is received, paths are decoded and queued into `PENDING_FILES`,
//! returning `noErr` (0) so macOS LaunchServices knows the operation succeeded.

use std::path::PathBuf;
use std::sync::Mutex;

static PENDING_FILES: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

/// Returns and removes the next pending file queued by macOS Finder / LaunchServices.
pub fn take_pending_file() -> Option<PathBuf> {
    PENDING_FILES.lock().ok()?.pop()
}

/// Queues a file path to be opened by the viewer.
#[allow(dead_code)]
pub fn queue_file(path: PathBuf) {
    if let Ok(mut lock) = PENDING_FILES.lock() {
        lock.push(path);
    }
}

/// Initializes macOS file event handlers so Finder's "Open With" delivers files properly.
#[cfg(target_os = "macos")]
pub fn init_macos_file_handler() {
    use std::ffi::{c_char, c_void, CStr};

    // --- 1. Apple Event Manager Handler ('aevt' / 'odoc') ---
    const K_CORE_EVENT_CLASS: u32 = 0x61657674; // 'aevt'
    const K_AE_OPEN_DOCUMENTS: u32 = 0x6f646f63; // 'odoc'
    const KEY_DIRECT_OBJECT: u32 = 0x2d2d2d2d; // '----'
    const TYPE_AE_LIST: u32 = 0x6c697374; // 'list'
    const TYPE_FILE_URL: u32 = 0x6675726c; // 'furl'

    #[repr(C)]
    struct AEDesc {
        _descriptor_type: u32,
        _data_handle: *mut c_void,
    }

    #[link(name = "CoreServices", kind = "framework")]
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn AEInstallEventHandler(
            the_ae_event_class: u32,
            the_ae_event_id: u32,
            handler: unsafe extern "C" fn(*const c_void, *mut c_void, isize) -> i16,
            handler_refcon: isize,
            is_sys_handler: bool,
        ) -> i16;

        fn AEGetParamDesc(
            the_apple_event: *const c_void,
            the_ae_keyword: u32,
            desired_type: u32,
            result: *mut AEDesc,
        ) -> i16;

        fn AECountItems(the_ae_desc_list: *const AEDesc, the_count: *mut isize) -> i16;

        fn AEGetNthPtr(
            the_ae_desc_list: *const AEDesc,
            index: isize,
            desired_type: u32,
            the_ae_keyword: *mut u32,
            type_code: *mut u32,
            data_ptr: *mut u8,
            maximum_size: isize,
            actual_size: *mut isize,
        ) -> i16;

        fn AEDisposeDesc(the_ae_desc: *mut AEDesc) -> i16;

        fn CFURLCreateWithBytes(
            allocator: *const c_void,
            url_bytes: *const u8,
            length: isize,
            encoding: u32,
            base_url: *const c_void,
        ) -> *mut c_void;

        fn CFURLCopyFileSystemPath(an_url: *const c_void, path_style: isize) -> *mut c_void;

        fn CFStringGetCString(
            the_string: *const c_void,
            buffer: *mut u8,
            buffer_size: isize,
            encoding: u32,
        ) -> bool;

        fn CFRelease(cf: *const c_void);
    }

    unsafe extern "C" fn handle_open_docs_apple_event(
        the_apple_event: *const c_void,
        _reply: *mut c_void,
        _refcon: isize,
    ) -> i16 {
        let mut doc_list = AEDesc {
            _descriptor_type: 0,
            _data_handle: std::ptr::null_mut(),
        };

        let err = AEGetParamDesc(
            the_apple_event,
            KEY_DIRECT_OBJECT,
            TYPE_AE_LIST,
            &mut doc_list,
        );

        if err == 0 {
            let mut count: isize = 0;
            if AECountItems(&doc_list, &mut count) == 0 {
                for i in 1..=count {
                    let mut keyword = 0u32;
                    let mut type_code = 0u32;
                    let mut buffer = [0u8; 4096];
                    let mut actual_size: isize = 0;

                    let get_err = AEGetNthPtr(
                        &doc_list,
                        i,
                        TYPE_FILE_URL,
                        &mut keyword,
                        &mut type_code,
                        buffer.as_mut_ptr(),
                        (buffer.len() - 1) as isize,
                        &mut actual_size,
                    );

                    if get_err == 0 && actual_size > 0 {
                        let len = (actual_size as usize).min(buffer.len() - 1);
                        let cf_url = CFURLCreateWithBytes(
                            std::ptr::null(),
                            buffer.as_ptr(),
                            len as isize,
                            0x0800_0100, // kCFStringEncodingUTF8
                            std::ptr::null(),
                        );
                        if !cf_url.is_null() {
                            let path_cf_str =
                                CFURLCopyFileSystemPath(cf_url, 0 /* kCFURLPOSIXPathStyle */);
                            if !path_cf_str.is_null() {
                                let mut path_buf = [0u8; 4096];
                                if CFStringGetCString(
                                    path_cf_str,
                                    path_buf.as_mut_ptr(),
                                    path_buf.len() as isize,
                                    0x0800_0100,
                                ) {
                                    if let Ok(path_str) =
                                        CStr::from_ptr(path_buf.as_ptr() as *const _).to_str()
                                    {
                                        queue_file(PathBuf::from(path_str));
                                    }
                                }
                                CFRelease(path_cf_str);
                            }
                            CFRelease(cf_url);
                        }
                    }
                }
            }
            AEDisposeDesc(&mut doc_list);
        }

        0 // noErr (signals successful handling to LaunchServices / Finder)
    }

    unsafe {
        AEInstallEventHandler(
            K_CORE_EVENT_CLASS,
            K_AE_OPEN_DOCUMENTS,
            handle_open_docs_apple_event,
            0,
            false,
        );
    }

    // --- 2. Objective-C AppKit Delegate Handlers on NSObject ---
    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn sel_registerName(name: *const c_char) -> *mut c_void;
        fn class_addMethod(
            cls: *mut c_void,
            name: *mut c_void,
            imp: *const c_void,
            types: *const c_char,
        ) -> bool;
        fn objc_msgSend();
    }

    unsafe {
        let ns_object = objc_getClass(b"NSObject\0".as_ptr() as *const _);
        if ns_object.is_null() {
            return;
        }

        // 2a. application:openFile: -> BOOL (B@:@@)
        unsafe extern "C" fn open_file_imp(
            _this: *mut c_void,
            _cmd: *mut c_void,
            _app: *mut c_void,
            path_str: *mut c_void,
        ) -> bool {
            if path_str.is_null() {
                return false;
            }
            let utf8_sel = sel_registerName(b"UTF8String\0".as_ptr() as *const _);
            let utf8_fn: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *const c_char =
                std::mem::transmute(objc_msgSend as *const ());
            let cstr = utf8_fn(path_str, utf8_sel);
            if !cstr.is_null() {
                if let Ok(path_slice) = CStr::from_ptr(cstr).to_str() {
                    let path = PathBuf::from(path_slice);
                    queue_file(path);
                    return true;
                }
            }
            false
        }

        let sel_open_file = sel_registerName(b"application:openFile:\0".as_ptr() as *const _);
        class_addMethod(
            ns_object,
            sel_open_file,
            open_file_imp as *const c_void,
            b"B@:@@\0".as_ptr() as *const _,
        );

        // 2b. application:openFiles: -> void (v@:@@)
        unsafe extern "C" fn open_files_imp(
            _this: *mut c_void,
            _cmd: *mut c_void,
            app: *mut c_void,
            files_arr: *mut c_void,
        ) {
            if files_arr.is_null() {
                return;
            }
            let count_sel = sel_registerName(b"count\0".as_ptr() as *const _);
            let count_fn: unsafe extern "C" fn(*mut c_void, *mut c_void) -> usize =
                std::mem::transmute(objc_msgSend as *const ());
            let count = count_fn(files_arr, count_sel);

            let at_index_sel = sel_registerName(b"objectAtIndex:\0".as_ptr() as *const _);
            let at_index_fn: unsafe extern "C" fn(*mut c_void, *mut c_void, usize) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            let utf8_sel = sel_registerName(b"UTF8String\0".as_ptr() as *const _);
            let utf8_fn: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *const c_char =
                std::mem::transmute(objc_msgSend as *const ());

            for i in 0..count {
                let ns_str = at_index_fn(files_arr, at_index_sel, i);
                if !ns_str.is_null() {
                    let cstr = utf8_fn(ns_str, utf8_sel);
                    if !cstr.is_null() {
                        if let Ok(path_slice) = CStr::from_ptr(cstr).to_str() {
                            queue_file(PathBuf::from(path_slice));
                        }
                    }
                }
            }

            if !app.is_null() {
                let reply_sel = sel_registerName(b"replyToOpenOrPrint:\0".as_ptr() as *const _);
                let reply_fn: unsafe extern "C" fn(*mut c_void, *mut c_void, usize) =
                    std::mem::transmute(objc_msgSend as *const ());
                reply_fn(app, reply_sel, 0); // NSApplicationDelegateReplySuccess = 0
            }
        }

        let sel_open_files = sel_registerName(b"application:openFiles:\0".as_ptr() as *const _);
        class_addMethod(
            ns_object,
            sel_open_files,
            open_files_imp as *const c_void,
            b"v@:@@\0".as_ptr() as *const _,
        );

        // 2c. application:openURLs: -> void (v@:@@)
        unsafe extern "C" fn open_urls_imp(
            _this: *mut c_void,
            _cmd: *mut c_void,
            app: *mut c_void,
            urls_arr: *mut c_void,
        ) {
            if urls_arr.is_null() {
                return;
            }
            let count_sel = sel_registerName(b"count\0".as_ptr() as *const _);
            let count_fn: unsafe extern "C" fn(*mut c_void, *mut c_void) -> usize =
                std::mem::transmute(objc_msgSend as *const ());
            let count = count_fn(urls_arr, count_sel);

            let at_index_sel = sel_registerName(b"objectAtIndex:\0".as_ptr() as *const _);
            let at_index_fn: unsafe extern "C" fn(*mut c_void, *mut c_void, usize) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            let path_sel = sel_registerName(b"path\0".as_ptr() as *const _);
            let path_fn: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            let utf8_sel = sel_registerName(b"UTF8String\0".as_ptr() as *const _);
            let utf8_fn: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *const c_char =
                std::mem::transmute(objc_msgSend as *const ());

            for i in 0..count {
                let url = at_index_fn(urls_arr, at_index_sel, i);
                if !url.is_null() {
                    let ns_str = path_fn(url, path_sel);
                    if !ns_str.is_null() {
                        let cstr = utf8_fn(ns_str, utf8_sel);
                        if !cstr.is_null() {
                            if let Ok(path_slice) = CStr::from_ptr(cstr).to_str() {
                                queue_file(PathBuf::from(path_slice));
                            }
                        }
                    }
                }
            }

            if !app.is_null() {
                let reply_sel = sel_registerName(b"replyToOpenOrPrint:\0".as_ptr() as *const _);
                let reply_fn: unsafe extern "C" fn(*mut c_void, *mut c_void, usize) =
                    std::mem::transmute(objc_msgSend as *const ());
                reply_fn(app, reply_sel, 0); // NSApplicationDelegateReplySuccess = 0
            }
        }

        let sel_open_urls = sel_registerName(b"application:openURLs:\0".as_ptr() as *const _);
        class_addMethod(
            ns_object,
            sel_open_urls,
            open_urls_imp as *const c_void,
            b"v@:@@\0".as_ptr() as *const _,
        );
    }
}

/// No-op implementation for non-macOS platforms.
#[cfg(not(target_os = "macos"))]
pub fn init_macos_file_handler() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_and_take_pending_file() {
        let test_path = PathBuf::from("/tmp/sample_image.jpg");
        queue_file(test_path.clone());
        assert_eq!(take_pending_file(), Some(test_path));
        assert_eq!(take_pending_file(), None);
    }

    #[test]
    fn test_init_macos_file_handler_idempotent() {
        init_macos_file_handler();
        init_macos_file_handler();
    }
}
