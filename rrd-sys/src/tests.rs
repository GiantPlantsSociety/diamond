use super::*;

#[test]
fn bindgen_test_layout_rrd_file_t() {
    assert_eq!(::std::mem::size_of::<rrd_file_t>(), 40usize, "Size of rrd_file_t");
    assert_eq!(::std::mem::align_of::<rrd_file_t>(), 8usize, "Alignment of rrd_file_t");
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_file_t>())).header_len as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_file_t),
            "::",
            stringify!(header_len)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_file_t>())).file_len as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_file_t),
            "::",
            stringify!(file_len)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_file_t>())).pos as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_file_t),
            "::",
            stringify!(pos)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_file_t>())).pvt as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_file_t),
            "::",
            stringify!(pvt)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_file_t>())).rrd as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_file_t),
            "::",
            stringify!(rrd)
        )
    );
}

#[test]
fn bindgen_test_layout_rrd_simple_file_t() {
    assert_eq!(::std::mem::size_of::<rrd_simple_file_t>(), 4usize, "Size of rrd_simple_file_t");
    assert_eq!(::std::mem::align_of::<rrd_simple_file_t>(), 4usize, "Alignment of rrd_simple_file_t");
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_simple_file_t>())).fd as *const _ as usize },
        0usize, "Offset of field rrd_simple_file_t::fd"
    );
}

#[test]
fn bindgen_test_layout_rrd_blob_t() {
    assert_eq!(
        ::std::mem::size_of::<rrd_blob_t>(),
        16usize,
        concat!("Size of: ", stringify!(rrd_blob_t))
    );
    assert_eq!(
        ::std::mem::align_of::<rrd_blob_t>(),
        8usize,
        concat!("Alignment of ", stringify!(rrd_blob_t))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_blob_t>())).size as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_blob_t),
            "::",
            stringify!(size)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_blob_t>())).ptr as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_blob_t),
            "::",
            stringify!(ptr)
        )
    );
}

#[test]
fn bindgen_test_layout_rrd_infoval() {
    assert_eq!(
        ::std::mem::size_of::<rrd_infoval>(),
        16usize,
        concat!("Size of: ", stringify!(rrd_infoval))
    );
    assert_eq!(
        ::std::mem::align_of::<rrd_infoval>(),
        8usize,
        concat!("Alignment of ", stringify!(rrd_infoval))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_infoval>())).u_cnt as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_infoval),
            "::",
            stringify!(u_cnt)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_infoval>())).u_val as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_infoval),
            "::",
            stringify!(u_val)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_infoval>())).u_str as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_infoval),
            "::",
            stringify!(u_str)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_infoval>())).u_int as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_infoval),
            "::",
            stringify!(u_int)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_infoval>())).u_blo as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_infoval),
            "::",
            stringify!(u_blo)
        )
    );
}

#[test]
fn bindgen_test_layout_rrd_info_t() {
    assert_eq!(
        ::std::mem::size_of::<rrd_info_t>(),
        40usize,
        concat!("Size of: ", stringify!(rrd_info_t))
    );
    assert_eq!(
        ::std::mem::align_of::<rrd_info_t>(),
        8usize,
        concat!("Alignment of ", stringify!(rrd_info_t))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_info_t>())).key as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_info_t),
            "::",
            stringify!(key)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_info_t>())).type_ as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_info_t),
            "::",
            stringify!(type_)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_info_t>())).value as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_info_t),
            "::",
            stringify!(value)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_info_t>())).next as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_info_t),
            "::",
            stringify!(next)
        )
    );
}

#[test]
fn bindgen_test_layout_rrd_time_value() {
    assert_eq!(
        ::std::mem::size_of::<rrd_time_value>(),
        72usize,
        concat!("Size of: ", stringify!(rrd_time_value))
    );
    assert_eq!(
        ::std::mem::align_of::<rrd_time_value>(),
        8usize,
        concat!("Alignment of ", stringify!(rrd_time_value))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_time_value>())).type_ as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_time_value),
            "::",
            stringify!(type_)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_time_value>())).offset as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_time_value),
            "::",
            stringify!(offset)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_time_value>())).tm as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_time_value),
            "::",
            stringify!(tm)
        )
    );
}

#[test]
fn bindgen_test_layout_rrd_context() {
    assert_eq!(
        ::std::mem::size_of::<rrd_context>(),
        4352usize,
        concat!("Size of: ", stringify!(rrd_context))
    );
    assert_eq!(
        ::std::mem::align_of::<rrd_context>(),
        1usize,
        concat!("Alignment of ", stringify!(rrd_context))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_context>())).lib_errstr as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_context),
            "::",
            stringify!(lib_errstr)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<rrd_context>())).rrd_error as *const _ as usize },
        256usize,
        concat!(
            "Offset of field: ",
            stringify!(rrd_context),
            "::",
            stringify!(rrd_error)
        )
    );
}
