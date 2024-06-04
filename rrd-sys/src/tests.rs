use super::*;
use std::mem::{align_of, offset_of, size_of};

#[test]
fn bindgen_test_layout_rrd_file_t() {
    assert_eq!(size_of::<rrd_file_t>(), 40);
    assert_eq!(align_of::<rrd_file_t>(), 8);
    assert_eq!(offset_of!(rrd_file_t, header_len), 0,);
    assert_eq!(offset_of!(rrd_file_t, file_len), 8,);
    assert_eq!(offset_of!(rrd_file_t, pos), 16,);
    assert_eq!(offset_of!(rrd_file_t, pvt), 24,);
    assert_eq!(offset_of!(rrd_file_t, rrd), 32);
}

#[test]
fn bindgen_test_layout_rrd_simple_file_t() {
    assert_eq!(size_of::<rrd_simple_file_t>(), 4,);
    assert_eq!(align_of::<rrd_simple_file_t>(), 4,);
    assert_eq!(offset_of!(rrd_simple_file_t, fd), 0,);
}

#[test]
fn bindgen_test_layout_rrd_blob_t() {
    assert_eq!(size_of::<rrd_blob_t>(), 16);
    assert_eq!(align_of::<rrd_blob_t>(), 8);
    assert_eq!(offset_of!(rrd_blob_t, size), 0);
    assert_eq!(offset_of!(rrd_blob_t, ptr), 8);
}

#[test]
fn bindgen_test_layout_rrd_infoval() {
    assert_eq!(size_of::<rrd_infoval>(), 16);
    assert_eq!(align_of::<rrd_infoval>(), 8);
    assert_eq!(offset_of!(rrd_infoval, u_cnt), 0);
    assert_eq!(offset_of!(rrd_infoval, u_val), 0);
    assert_eq!(offset_of!(rrd_infoval, u_str), 0);
    assert_eq!(offset_of!(rrd_infoval, u_int), 0);
    assert_eq!(offset_of!(rrd_infoval, u_blo), 0);
}

#[test]
fn bindgen_test_layout_rrd_info_t() {
    assert_eq!(size_of::<rrd_info_t>(), 40,);
    assert_eq!(align_of::<rrd_info_t>(), 8);
    assert_eq!(offset_of!(rrd_info_t, key), 0);
    assert_eq!(offset_of!(rrd_info_t, type_), 8);
    assert_eq!(offset_of!(rrd_info_t, value), 16);
    assert_eq!(offset_of!(rrd_info_t, next), 32);
}

#[test]
fn bindgen_test_layout_rrd_time_value() {
    assert_eq!(size_of::<rrd_time_value>(), 72);
    assert_eq!(align_of::<rrd_time_value>(), 8);
    assert_eq!(offset_of!(rrd_time_value, type_), 0);
    assert_eq!(offset_of!(rrd_time_value, offset), 8);
    assert_eq!(offset_of!(rrd_time_value, tm), 16);
}

#[test]
fn bindgen_test_layout_rrd_context() {
    assert_eq!(size_of::<rrd_context>(), 4352);
    assert_eq!(align_of::<rrd_context>(), 1);
    assert_eq!(offset_of!(rrd_context, lib_errstr), 0);
    assert_eq!(offset_of!(rrd_context, rrd_error), 256);
}
