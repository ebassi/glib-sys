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
use gstr;
use types::gchar;

use std::mem;
use std::str;
use std::sync::atomic;

#[deriving(Copy, Eq, PartialEq)]
pub struct Quark(ffi::GQuark);

pub struct StaticQuark(pub &'static [u8], pub atomic::AtomicUint);

impl Quark {

    pub unsafe fn new(raw: ffi::GQuark) -> Quark {
        Quark(raw)
    }

    pub fn from_static_str(s: &'static str) -> Quark {
        debug_assert!(s.ends_with("\0"));
        unsafe {
            let p = s.as_ptr() as *const gchar;
            Quark::new(ffi::g_quark_from_static_string(p))
        }
    }

    pub fn to_uint(&self) -> uint {
        let Quark(raw) = *self;
        raw as uint
    }

    pub fn to_bytes(&self) -> &'static [u8] {
        let Quark(raw) = *self;
        unsafe {
            let s = ffi::g_quark_to_string(raw);
            let r = mem::copy_lifetime("", &s);
            gstr::parse_as_bytes(r)
        }
    }

    #[inline]
    pub fn to_str(&self) -> Result<&'static str, str::Utf8Error> {
        str::from_utf8(self.to_bytes())
    }
}

impl StaticQuark {

    pub fn get(&self) -> Quark {
        let StaticQuark(s, ref cached) = *self;
        let mut q = cached.load(atomic::Ordering::Relaxed);
        if q == 0 {
            unsafe {
                let p = s.as_ptr() as *const gchar;
                q = ffi::g_quark_from_static_string(p) as uint;
            }
            cached.store(q, atomic::Ordering::Relaxed);
        }
        unsafe { Quark::new(q as ffi::GQuark) }
    }
}
