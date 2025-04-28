#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libc;

use libc::{FILE, mode_t, time_t, tm};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rrd_t {
    _unused: [u8; 0],
}

extern "C" {
    pub fn rrd_set_to_DNAN() -> f64;
    pub fn rrd_set_to_DINF() -> f64;
}

pub type rrd_value_t = f64;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rrd_file_t {
    pub header_len: usize,
    pub file_len: usize,
    pub pos: usize,
    pub pvt: *mut ::std::os::raw::c_void,
    pub rrd: *mut rrd_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rrd_simple_file_t {
    pub fd: ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rrd_blob_t {
    pub size: ::std::os::raw::c_ulong,
    pub ptr: *mut ::std::os::raw::c_uchar,
}

pub const rrd_info_type_RD_I_VAL: rrd_info_type = 0;
pub const rrd_info_type_RD_I_CNT: rrd_info_type = 1;
pub const rrd_info_type_RD_I_STR: rrd_info_type = 2;
pub const rrd_info_type_RD_I_INT: rrd_info_type = 3;
pub const rrd_info_type_RD_I_BLO: rrd_info_type = 4;

pub type rrd_info_type = u32;
pub use self::rrd_info_type as rrd_info_type_t;
#[repr(C)]
#[derive(Copy, Clone)]
pub union rrd_infoval {
    pub u_cnt: ::std::os::raw::c_ulong,
    pub u_val: rrd_value_t,
    pub u_str: *mut ::std::os::raw::c_char,
    pub u_int: ::std::os::raw::c_int,
    pub u_blo: rrd_blob_t,
    _bindgen_union_align: [u64; 2usize],
}

pub type rrd_infoval_t = rrd_infoval;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct rrd_info_t {
    pub key: *mut ::std::os::raw::c_char,
    pub type_: rrd_info_type_t,
    pub value: rrd_infoval_t,
    pub next: *mut rrd_info_t,
}

pub type rrd_output_callback_t = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *const ::std::os::raw::c_void,
        arg2: usize,
        arg3: *mut ::std::os::raw::c_void,
    ) -> usize,
>;

extern "C" {
    pub fn rrd_create(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_info(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> *mut rrd_info_t;
    pub fn rrd_info_push(
        arg1: *mut rrd_info_t,
        arg2: *mut ::std::os::raw::c_char,
        arg3: rrd_info_type_t,
        arg4: rrd_infoval_t,
    ) -> *mut rrd_info_t;
    pub fn rrd_info_print(data: *mut rrd_info_t);
    pub fn rrd_info_free(arg1: *mut rrd_info_t);
    pub fn rrd_list(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;
    pub fn rrd_list_r(
        arg1: ::std::os::raw::c_int,
        dirname: *mut ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;
    pub fn rrd_update(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_update_v(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> *mut rrd_info_t;
    pub fn rrd_graph(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
        arg3: *mut *mut *mut ::std::os::raw::c_char,
        arg4: *mut ::std::os::raw::c_int,
        arg5: *mut ::std::os::raw::c_int,
        arg6: *mut FILE,
        arg7: *mut f64,
        arg8: *mut f64,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_graph_v(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> *mut rrd_info_t;
    pub fn rrd_fetch(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
        arg3: *mut time_t,
        arg4: *mut time_t,
        arg5: *mut ::std::os::raw::c_ulong,
        arg6: *mut ::std::os::raw::c_ulong,
        arg7: *mut *mut *mut ::std::os::raw::c_char,
        arg8: *mut *mut rrd_value_t,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_restore(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_dump(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_tune(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_last(arg1: ::std::os::raw::c_int, arg2: *mut *mut ::std::os::raw::c_char) -> time_t;
    pub fn rrd_lastupdate(
        argc: ::std::os::raw::c_int,
        argv: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_first(arg1: ::std::os::raw::c_int, arg2: *mut *mut ::std::os::raw::c_char)
    -> time_t;
    pub fn rrd_resize(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_strversion() -> *mut ::std::os::raw::c_char;
    pub fn rrd_version() -> f64;
    pub fn rrd_xport(
        arg1: ::std::os::raw::c_int,
        arg2: *mut *mut ::std::os::raw::c_char,
        arg3: *mut ::std::os::raw::c_int,
        arg4: *mut time_t,
        arg5: *mut time_t,
        arg6: *mut ::std::os::raw::c_ulong,
        arg7: *mut ::std::os::raw::c_ulong,
        arg8: *mut *mut *mut ::std::os::raw::c_char,
        arg9: *mut *mut rrd_value_t,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_flushcached(
        argc: ::std::os::raw::c_int,
        argv: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_freemem(mem: *mut ::std::os::raw::c_void);
    pub fn rrd_create_r(
        filename: *const ::std::os::raw::c_char,
        pdp_step: ::std::os::raw::c_ulong,
        last_up: time_t,
        argc: ::std::os::raw::c_int,
        argv: *mut *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_create_r2(
        filename: *const ::std::os::raw::c_char,
        pdp_step: ::std::os::raw::c_ulong,
        last_up: time_t,
        no_overwrite: ::std::os::raw::c_int,
        sources: *mut *const ::std::os::raw::c_char,
        _template: *const ::std::os::raw::c_char,
        argc: ::std::os::raw::c_int,
        argv: *mut *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_info_r(arg1: *const ::std::os::raw::c_char) -> *mut rrd_info_t;
    pub fn rrd_update_r(
        filename: *const ::std::os::raw::c_char,
        _template: *const ::std::os::raw::c_char,
        argc: ::std::os::raw::c_int,
        argv: *mut *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_update_v_r(
        filename: *const ::std::os::raw::c_char,
        _template: *const ::std::os::raw::c_char,
        argc: ::std::os::raw::c_int,
        argv: *mut *const ::std::os::raw::c_char,
        pcdp_summary: *mut rrd_info_t,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_updatex_r(
        filename: *const ::std::os::raw::c_char,
        _template: *const ::std::os::raw::c_char,
        extra_flags: ::std::os::raw::c_int,
        argc: ::std::os::raw::c_int,
        argv: *mut *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_updatex_v_r(
        filename: *const ::std::os::raw::c_char,
        _template: *const ::std::os::raw::c_char,
        extra_flags: ::std::os::raw::c_int,
        argc: ::std::os::raw::c_int,
        argv: *mut *const ::std::os::raw::c_char,
        pcdp_summary: *mut rrd_info_t,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_fetch_r(
        filename: *const ::std::os::raw::c_char,
        cf: *const ::std::os::raw::c_char,
        start: *mut time_t,
        end: *mut time_t,
        step: *mut ::std::os::raw::c_ulong,
        ds_cnt: *mut ::std::os::raw::c_ulong,
        ds_namv: *mut *mut *mut ::std::os::raw::c_char,
        data: *mut *mut rrd_value_t,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_dump_r(
        filename: *const ::std::os::raw::c_char,
        outname: *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_last_r(filename: *const ::std::os::raw::c_char) -> time_t;
    pub fn rrd_lastupdate_r(
        filename: *const ::std::os::raw::c_char,
        ret_last_update: *mut time_t,
        ret_ds_count: *mut ::std::os::raw::c_ulong,
        ret_ds_names: *mut *mut *mut ::std::os::raw::c_char,
        ret_last_ds: *mut *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_first_r(
        filename: *const ::std::os::raw::c_char,
        rraindex: ::std::os::raw::c_int,
    ) -> time_t;
    pub fn rrd_dump_cb_r(
        filename: *const ::std::os::raw::c_char,
        opt_header: ::std::os::raw::c_int,
        cb: rrd_output_callback_t,
        user: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
pub const rrd_timetype_t_ABSOLUTE_TIME: rrd_timetype_t = 0;
pub const rrd_timetype_t_RELATIVE_TO_START_TIME: rrd_timetype_t = 1;
pub const rrd_timetype_t_RELATIVE_TO_END_TIME: rrd_timetype_t = 2;
pub const rrd_timetype_t_RELATIVE_TO_EPOCH: rrd_timetype_t = 3;
pub type rrd_timetype_t = u32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct rrd_time_value {
    pub type_: rrd_timetype_t,
    pub offset: ::std::os::raw::c_long,
    pub tm: tm,
}

pub type rrd_time_value_t = rrd_time_value;

extern "C" {
    pub fn rrd_parsetime(
        spec: *const ::std::os::raw::c_char,
        ptv: *mut rrd_time_value_t,
    ) -> *mut ::std::os::raw::c_char;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct rrd_context {
    pub lib_errstr: [::std::os::raw::c_char; 256usize],
    pub rrd_error: [::std::os::raw::c_char; 4096usize],
}

pub type rrd_context_t = rrd_context;

extern "C" {
    pub fn rrd_get_context() -> *mut rrd_context_t;
    pub fn rrd_proc_start_end(
        arg1: *mut rrd_time_value_t,
        arg2: *mut rrd_time_value_t,
        arg3: *mut time_t,
        arg4: *mut time_t,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_set_error(arg1: *mut ::std::os::raw::c_char, ...);
    pub fn rrd_clear_error();
    pub fn rrd_test_error() -> ::std::os::raw::c_int;
    pub fn rrd_get_error() -> *mut ::std::os::raw::c_char;
    pub fn rrd_strerror(err: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char;
    pub fn rrd_new_context() -> *mut rrd_context_t;
    pub fn rrd_free_context(buf: *mut rrd_context_t);
    pub fn rrd_random() -> ::std::os::raw::c_long;
    pub fn rrd_add_ptr_chunk(
        dest: *mut *mut *mut ::std::os::raw::c_void,
        dest_size: *mut usize,
        src: *mut ::std::os::raw::c_void,
        alloc: *mut usize,
        chunk: usize,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_add_ptr(
        dest: *mut *mut *mut ::std::os::raw::c_void,
        dest_size: *mut usize,
        src: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_add_strdup(
        dest: *mut *mut *mut ::std::os::raw::c_char,
        dest_size: *mut usize,
        src: *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_add_strdup_chunk(
        dest: *mut *mut *mut ::std::os::raw::c_char,
        dest_size: *mut usize,
        src: *mut ::std::os::raw::c_char,
        alloc: *mut usize,
        chunk: usize,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_free_ptrs(src: *mut *mut *mut ::std::os::raw::c_void, cnt: *mut usize);
    pub fn rrd_mkdir_p(
        pathname: *const ::std::os::raw::c_char,
        mode: mode_t,
    ) -> ::std::os::raw::c_int;
    pub fn rrd_scaled_duration(
        token: *const ::std::os::raw::c_char,
        divisor: ::std::os::raw::c_ulong,
        valuep: *mut ::std::os::raw::c_ulong,
    ) -> *const ::std::os::raw::c_char;
}

#[cfg(test)]
mod tests;
