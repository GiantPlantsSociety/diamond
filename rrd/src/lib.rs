#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate num_traits;
/// https://oss.oetiker.ch/rrdtool/doc/rrdcreate.en.html
extern crate regex;
extern crate rrd_sys;

use num_traits::cast;
use regex::Regex;
use rrd_sys::*;
use std::collections::{BTreeSet, HashMap};
use std::ffi::{CStr, CString};
use std::fmt;
use std::marker::PhantomData;
use std::ops::Drop;
use std::os::raw::{c_char, c_long, c_ulong, c_void};
use std::path::Path;
use std::ptr;
use std::str::FromStr;

#[derive(Debug)]
pub struct Error(String);

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum Value {
    Float(f64),
    Long(u64),
    Int(i32),
    Text(String),
    Blob(Vec<u8>),
}

impl Value {
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(x) => Some(*x),
            Value::Long(x) => Some(*x as f64),
            Value::Int(x) => Some(f64::from(*x)),
            Value::Text(x) => x.parse().ok(),
            _ => None,
        }
    }

    pub fn as_long(&self) -> Option<u64> {
        match self {
            Value::Long(x) => Some(*x),
            Value::Int(x) => cast(*x),
            Value::Text(x) => x.parse().ok(),
            _ => None,
        }
    }

    pub fn int(&self) -> Option<i32> {
        match self {
            Value::Long(x) => cast(*x),
            Value::Int(x) => Some(*x),
            Value::Text(x) => x.parse().ok(),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Value::Text(x) => Some(x),
            _ => None,
        }
    }

    pub fn blob(&self) -> Option<&[u8]> {
        match self {
            Value::Blob(x) => Some(x),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AggregationMethod {
    Average,
    Last,
    Max,
    Min,
}

impl FromStr for AggregationMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("average") {
            Ok(AggregationMethod::Average)
        } else if s.eq_ignore_ascii_case("min") {
            Ok(AggregationMethod::Min)
        } else if s.eq_ignore_ascii_case("max") {
            Ok(AggregationMethod::Max)
        } else if s.eq_ignore_ascii_case("last") {
            Ok(AggregationMethod::Last)
        } else {
            Err(format!("Unsupported aggregation method '{}'.", s))
        }
    }
}

impl From<AggregationMethod> for &'static str {
    fn from(val: AggregationMethod) -> Self {
        match val {
            AggregationMethod::Average => "average",
            AggregationMethod::Min => "min",
            AggregationMethod::Max => "max",
            AggregationMethod::Last => "last",
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RRA {
    pub cf: AggregationMethod,
    pub rows: u64,
    pub cur_row: u64,
    pub pdp_per_row: u64,
    pub xff: f64,
}

#[derive(Debug)]
pub struct Info(*mut rrd_info_t);

impl Info {
    pub fn iter(&self) -> InfoIter {
        InfoIter(self.0, PhantomData)
    }

    pub fn rra_count(&self) -> usize {
        let re = Regex::new("^rra\\[(\\d+)\\]").unwrap();
        self.iter()
            .filter_map(|(key, _value)| {
                if let Some(m) = re.captures(&key) {
                    m.get(1).unwrap().as_str().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .max()
            .map_or(0, |c| c + 1)
    }

    pub fn rras(&self) -> Vec<RRA> {
        let hash: HashMap<String, Value> = self.iter().collect();

        (0..self.rra_count())
            .map(|index| RRA {
                cf: AggregationMethod::from_str(
                    hash[&format!("rra[{}].cf", index)].text().unwrap(),
                )
                .unwrap(),
                rows: hash[&format!("rra[{}].rows", index)].as_long().unwrap(),
                cur_row: hash[&format!("rra[{}].cur_row", index)].as_long().unwrap(),
                pdp_per_row: hash[&format!("rra[{}].pdp_per_row", index)]
                    .as_long()
                    .unwrap(),
                xff: hash[&format!("rra[{}].xff", index)].as_float().unwrap(),
            })
            .collect()
    }

    pub fn datasources(&self) -> BTreeSet<String> {
        let re = Regex::new("^ds\\[([^\\]]+)\\]").unwrap();
        self.iter()
            .filter_map(|(key, _value)| {
                re.captures(&key)
                    .and_then(|c| c.get(1))
                    .map(|c| c.as_str().to_owned())
            })
            .collect()
    }
}

pub struct InfoIter<'a>(*const rrd_info_t, PhantomData<&'a ()>);

impl Iterator for InfoIter<'_> {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_null() {
            None
        } else {
            let info = self.0;

            let key = unsafe { CStr::from_ptr((*info).key) }
                .to_str()
                .unwrap()
                .to_owned();

            let value = match unsafe { (*info).type_ } {
                rrd_info_type_RD_I_VAL => Value::Float(unsafe { (*info).value.u_val }),
                rrd_info_type_RD_I_CNT => Value::Long(unsafe { (*info).value.u_cnt }),
                rrd_info_type_RD_I_INT => Value::Int(unsafe { (*info).value.u_int }),
                rrd_info_type_RD_I_STR => {
                    let cstr = unsafe { CStr::from_ptr((*info).value.u_str) };
                    let text = cstr.to_str().unwrap().to_owned();
                    Value::Text(text)
                }
                rrd_info_type_RD_I_BLO => {
                    let block = unsafe { (*info).value.u_blo };
                    let buffer = vec![0; block.size as usize];
                    unsafe {
                        ptr::copy_nonoverlapping(
                            block.ptr,
                            buffer.as_ptr() as *mut u8,
                            block.size as usize,
                        )
                    };
                    Value::Blob(buffer)
                }
                t => panic!("Unknown type of info value {}", t),
            };
            self.0 = unsafe { (*self.0).next };
            Some((key, value))
        }
    }
}

impl Drop for Info {
    fn drop(&mut self) {
        unsafe {
            rrd_info_free(self.0);
        }
    }
}

fn get_and_clear_error() -> Error {
    unsafe {
        let c_err = rrd_get_error();
        let error = Error(
            CStr::from_ptr(c_err)
                .to_str()
                .map_err(|e| Error(e.to_string()))
                .unwrap()
                .to_owned(),
        );
        rrd_clear_error();
        error
    }
}

pub fn info(filename: &Path, daemon: Option<&Path>, noflush: bool) -> Result<Info, Error> {
    let mut c_args = Vec::<*const c_char>::new();

    let info_str = CString::new("info").unwrap();
    c_args.push(info_str.as_ptr());

    let c_filename = CString::new(filename.to_str().unwrap().as_bytes()).unwrap();
    c_args.push(c_filename.as_ptr());

    if let Some(daemon_path) = daemon {
        let daemon_str = CString::new("--daemon").unwrap();
        c_args.push(daemon_str.as_ptr());
        let c_daemon_path = CString::new(daemon_path.to_str().unwrap().as_bytes()).unwrap();
        c_args.push(c_daemon_path.as_ptr());
    }

    if noflush {
        let noflush_str = CString::new("--noflush").unwrap();
        c_args.push(noflush_str.as_ptr());
    }

    let info = unsafe { rrd_info(c_args.len() as i32, c_args.as_ptr() as *mut *mut c_char) };

    if info.is_null() {
        Err(get_and_clear_error())
    } else {
        Ok(Info(info))
    }
}

pub struct Range {
    pub start: i64,
    pub end: i64,
    pub step: u64,
}

pub struct Data {
    pub time_info: Range,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<f64>>,
}

pub fn fetch(
    filename: &Path,
    aggregation: AggregationMethod,
    resolution: Option<u32>,
    start: u64,
    end: u64,
) -> Result<Data, Error> {
    let c_filename = CString::new(filename.to_str().unwrap().as_bytes()).unwrap();
    let c_aggregation =
        CString::new(str::to_ascii_uppercase(aggregation.into()).as_bytes()).unwrap();

    let mut start = start as c_long;
    let mut end = end as c_long;
    let mut step = u64::from(resolution.unwrap_or(1)) as c_ulong;
    let mut ds_cnt = 0;
    let mut ds_namv = ptr::null_mut();
    let mut data = ptr::null_mut();
    let status = unsafe {
        rrd_fetch_r(
            c_filename.as_ptr(),
            c_aggregation.as_ptr(),
            &mut start,
            &mut end,
            &mut step,
            &mut ds_cnt,
            &mut ds_namv,
            &mut data,
        )
    };

    if status == -1 {
        Err(get_and_clear_error())
    } else {
        let row = (end - start) / (step as i64);

        let mut columns = Vec::with_capacity(ds_cnt as usize);
        for i in 0..ds_cnt {
            let ptr = unsafe { *ds_namv.offset(i as isize) };
            let s = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_owned();
            columns.push(s);
            unsafe { rrd_freemem(ptr as *mut c_void) };
        }
        unsafe { rrd_freemem(ds_namv as *mut c_void) };

        let mut data_list = Vec::with_capacity(row as usize);
        for i in 0..row {
            let mut row = Vec::with_capacity(ds_cnt as usize);
            for j in 0..ds_cnt {
                let dv = unsafe { *data.offset((i * (ds_cnt as i64) + (j as i64)) as isize) };
                row.push(dv);
            }
            data_list.push(row);
        }
        unsafe { rrd_freemem(data as *mut c_void) };

        Ok(Data {
            time_info: Range { start, end, step },
            columns,
            rows: data_list,
        })
    }
}
