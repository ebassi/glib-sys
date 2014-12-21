// This file is part of Grust, GObject introspection bindings for Rust
//
// Copyright (C) 2014  Mikhail Zabaluev <mikhail.zabaluev@gmail.com>
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA

use ffi;
use quark::Quark;
use types::{gint,gpointer,gsize,gssize};
use util::is_true;

use std::default::Default;
use std::ptr;
use std::slice;
use std::str;

use libc;

pub mod raw {

    use ffi;
    use types::{gint,gchar};
    use std::kinds::marker;

    #[repr(C)]
    pub struct GError {
        pub domain: ffi::GQuark,
        pub code: gint,
        pub message: *const gchar,
        no_copy: marker::NoCopy
    }
}

pub struct Error {
    ptr: *mut raw::GError
}

pub fn unset() -> Error {
    Error { ptr: ptr::null_mut() }
}

pub trait ErrorDomain {
    fn error_domain(_: Option<Self>) -> Quark;
}

pub enum ErrorMatch<T> {
    NotInDomain,
    Known(T),
    Unknown(int)
}

impl Drop for Error {
    fn drop(&mut self) {
        if self.ptr.is_not_null() {
            unsafe { ffi::g_error_free(self.ptr); }
        }
    }
}

impl Clone for Error {
    fn clone(&self) -> Error {
        if self.ptr.is_null() {
            unset()
        } else {
            unsafe {
                Error { ptr: ffi::g_error_copy(self.ptr as *const raw::GError) }
            }
        }
    }
}

impl Default for Error {
    fn default() -> Error { unset() }
}

impl Error {
    pub unsafe fn slot_ptr(&mut self) -> *mut *mut raw::GError {
        &mut self.ptr as *mut *mut raw::GError
    }

    pub fn is_set(&self) -> bool { self.ptr.is_not_null() }

    pub fn key(&self) -> (Quark, int) {
        if self.ptr.is_null() {
            panic!("use of an unset GError pointer slot");
        }
        unsafe { ((*self.ptr).domain, (*self.ptr).code as int) }
    }

    pub fn message(&self) -> String {

        if self.ptr.is_null() {
            return String::from_str("no error");
        }

        // GError messages may come in any shape or form, but the best guesses
        // at the encoding would be: 1) UTF-8; 2) the locale encoding.

        unsafe {
            let raw_msg = (*self.ptr).message as *const u8;
            assert!(raw_msg.is_not_null());
            let len = libc::strlen(raw_msg as *const libc::c_char) as uint;
            let msg_bytes = slice::from_raw_buf(&raw_msg, len);

            match str::from_utf8(msg_bytes) {
                Some(s) => { return String::from_str(s); }
                None    => {}
            }

            let mut bytes_read: gsize = 0;
            let mut bytes_conv: gsize = 0;
            let conv_msg = ffi::g_locale_to_utf8(
                            raw_msg as *const libc::c_char,
                            len as gssize,
                            &mut bytes_read as *mut gsize,
                            &mut bytes_conv as *mut gsize,
                            ptr::null_mut());
            if conv_msg.is_not_null() {
                if bytes_read as uint == len {
                    let res = String::from_raw_buf_len(
                            conv_msg as *const u8, bytes_conv as uint);
                    ffi::g_free(conv_msg as gpointer);
                    return res;
                } else {
                    ffi::g_free(conv_msg as gpointer);
                }
            }

            // As the last resort, try to salvage what we can
            String::from_utf8_lossy(msg_bytes).into_string()
        }
    }

    pub fn matches<E: ErrorDomain + ToPrimitive + Copy>(&self, expected: E)
                    -> bool {
        if self.ptr.is_null() {
            panic!("use of an unset GError pointer slot");
        }
        let domain = ErrorDomain::error_domain(Some(expected));
        let code = expected.to_int().unwrap() as gint;
        unsafe {
            is_true(ffi::g_error_matches(self.ptr as *const raw::GError,
                                         domain, code))
        }
    }

    pub fn to_domain<E: ErrorDomain + FromPrimitive>(&self) -> ErrorMatch<E> {
        let (domain, code) = self.key();
        if domain != ErrorDomain::error_domain(None::<E>) {
            return ErrorMatch::NotInDomain;
        }
        let maybe_enum: Option<E> = FromPrimitive::from_int(code);
        match maybe_enum {
            Some(m) => ErrorMatch::Known(m),
            None    => ErrorMatch::Unknown(code)
        }
    }
}
