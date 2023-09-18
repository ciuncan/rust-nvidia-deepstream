#![allow(dead_code)]
#![allow(improper_ctypes)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(clippy::missing_safety_doc)]

pub use glib_sys::*;
pub use gobject_sys::*;

// TODO: Find a way to obtain these typedefs from somewhere
pub type gchar = ::std::os::raw::c_char;
pub type gshort = ::std::os::raw::c_short;
pub type glong = ::std::os::raw::c_long;
pub type gint = ::std::os::raw::c_int;
pub type guchar = ::std::os::raw::c_uchar;
pub type gushort = ::std::os::raw::c_ushort;
pub type gulong = ::std::os::raw::c_ulong;
pub type guint = ::std::os::raw::c_uint;
pub type gfloat = f32;
pub type gdouble = f64;

pub type gint8 = ::std::os::raw::c_schar;
pub type guint8 = ::std::os::raw::c_uchar;
pub type gint16 = ::std::os::raw::c_short;
pub type guint16 = ::std::os::raw::c_ushort;
pub type gint32 = ::std::os::raw::c_int;
pub type guint32 = ::std::os::raw::c_uint;
pub type gint64 = ::std::os::raw::c_long;
pub type guint64 = ::std::os::raw::c_ulong;
pub type gssize = ::std::os::raw::c_long;
pub type gsize = ::std::os::raw::c_ulong;
pub type goffset = gint64;
pub type gintptr = ::std::os::raw::c_long;
pub type guintptr = ::std::os::raw::c_ulong;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
