// Copyright 2022 foundationdb-rs developers, https://github.com/Clikengo/foundationdb-rs/graphs/contributors
// Copyright 2013-2018 Apple, Inc and the FoundationDB project authors.
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Definitions of MappedKeyValues, used in api version 710 and more.
//!
//! GetMappedRange is an experimental feature introduced in FDB 7.1. It is intended to improve the
//! client throughput and reduce latency for a commonly used traffic pattern.
//! An experiment with Record Layer shows that this optimization can get 4x client throughput on a certain workload.
//!
//! More info can be found in the [relevant documentation](https://github.com/apple/foundationdb/wiki/Everything-about-GetMappedRange).

use crate::future::{FdbFutureHandle, FdbKeyValue};
use crate::mem::read_unaligned_slice;
use crate::{error, KeySelector};
use crate::{FdbError, FdbResult};
use foundationdb_sys as fdb_sys;
use std::borrow::Cow;
use std::fmt;

use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::sync::Arc;

/// An slice of mapped keyvalues owned by a foundationDB future produced by the `get_mapped` method.
pub struct MappedKeyValues {
    _f: FdbFutureHandle,
    mapped_keyvalues: Vec<FdbMappedKeyValue>,
    more: bool,
}
unsafe impl Sync for MappedKeyValues {}
unsafe impl Send for MappedKeyValues {}

impl MappedKeyValues {
    /// `true` if there is another range after this one
    pub fn more(&self) -> bool {
        self.more
    }
}

impl TryFrom<FdbFutureHandle> for MappedKeyValues {
    type Error = FdbError;
    fn try_from(f: FdbFutureHandle) -> FdbResult<Self> {
        let mut keyvalues = std::ptr::null();
        let mut len = 0;
        let mut more = 0;

        unsafe {
            error::eval(fdb_sys::fdb_future_get_mappedkeyvalue_array(
                f.as_ptr(),
                &mut keyvalues,
                &mut len,
                &mut more,
            ))?
        }

        Ok(MappedKeyValues {
            _f: f,
            mapped_keyvalues: unsafe { read_unaligned_slice(keyvalues as *const _, len) },
            more: more != 0,
        })
    }
}

#[repr(transparent)]
/// A KeyValue produced by a mapped operation, ownder by a Foundation Future.
pub struct FdbMappedKeyValue(fdb_sys::FDBMappedKeyValue);

impl PartialEq for FdbMappedKeyValue {
    fn eq(&self, other: &Self) -> bool {
        (self.parent_key(), self.parent_value()) == (other.parent_key(), other.parent_value())
    }
}
impl Eq for FdbMappedKeyValue {}
impl fmt::Debug for FdbMappedKeyValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({:?}, {:?})",
            crate::tuple::Bytes::from(self.parent_key()),
            crate::tuple::Bytes::from(self.parent_value())
        )
    }
}

impl FdbMappedKeyValue {
    /// Retrieves the "parent" key that generated the secondary scan.
    pub fn parent_key(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.0.key.key as *const u8, self.0.key.key_length as usize)
        }
    }

    /// Retrieves the "parent" value that generated the secondary scan.
    pub fn parent_value(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.value.key as *const u8,
                self.0.value.key_length as usize,
            )
        }
    }

    /// Retrieves the beginning of the range
    pub fn begin_range(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.getRange.begin.key.key as *const u8,
                self.0.getRange.begin.key.key_length as usize,
            )
        }
    }

    /// Retrieves the end of the range
    pub fn end_range(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.getRange.end.key.key as *const u8,
                self.0.getRange.end.key.key_length as usize,
            )
        }
    }

    /// Retrieves the beginning of the range as a [`KeySelector`]
    pub fn begin_selector(&self) -> KeySelector {
        KeySelector::new(Cow::from(self.begin_range()), false, 0)
    }

    /// Retrieves the end of the range as a [`KeySelector`]
    pub fn end_selector(&self) -> KeySelector {
        KeySelector::new(Cow::from(self.end_range()), false, 0)
    }

    /// retrieves the associated slice of [`FdbKeyValue`]
    pub fn key_values(&self) -> Vec<FdbKeyValue> {
        unsafe {
            read_unaligned_slice(
                self.0.getRange.data as *const FdbKeyValue,
                self.0.getRange.m_size,
            )
        }
    }
}

impl Deref for MappedKeyValues {
    type Target = [FdbMappedKeyValue];

    fn deref(&self) -> &Self::Target {
        &self.mapped_keyvalues
    }
}

impl AsRef<[FdbMappedKeyValue]> for MappedKeyValues {
    fn as_ref(&self) -> &[FdbMappedKeyValue] {
        self.deref()
    }
}

impl<'a> IntoIterator for &'a MappedKeyValues {
    type Item = &'a FdbMappedKeyValue;
    type IntoIter = std::slice::Iter<'a, FdbMappedKeyValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().iter()
    }
}

/// An FdbMappedValue that you can own.
pub struct FdbMappedValue {
    _f: Arc<FdbFutureHandle>,
    mapped_keyvalue: FdbMappedKeyValue,
}

impl IntoIterator for MappedKeyValues {
    type Item = FdbMappedValue;
    type IntoIter = FdbMappedValuesIter;

    fn into_iter(self) -> Self::IntoIter {
        let mapped_keyvalues = ManuallyDrop::new(self.mapped_keyvalues);
        FdbMappedValuesIter {
            f: Arc::new(self._f),
            ptr: mapped_keyvalues.as_ptr(),
            len: mapped_keyvalues.len(),
            cap: mapped_keyvalues.capacity(),
            pos: 0,
        }
    }
}

unsafe impl Send for FdbMappedValue {}

impl Deref for FdbMappedValue {
    type Target = FdbMappedKeyValue;
    fn deref(&self) -> &Self::Target {
        &self.mapped_keyvalue
    }
}
impl AsRef<FdbMappedKeyValue> for FdbMappedValue {
    fn as_ref(&self) -> &FdbMappedKeyValue {
        self.deref()
    }
}
impl PartialEq for FdbMappedValue {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl Eq for FdbMappedValue {}

/// An iterator of mapped keyvalues owned by a foundationDB future
pub struct FdbMappedValuesIter {
    f: Arc<FdbFutureHandle>,
    ptr: *const FdbMappedKeyValue,
    len: usize,
    cap: usize,
    pos: usize,
}

unsafe impl Send for FdbMappedValuesIter {}

impl Iterator for FdbMappedValuesIter {
    type Item = FdbMappedValue;
    fn next(&mut self) -> Option<Self::Item> {
        #[allow(clippy::iter_nth_zero)]
        self.nth(0)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n < self.len - self.pos {
            // safe because pos < self.len
            let mapped_keyvalue = unsafe { self.ptr.add(self.pos + n).read() };
            self.pos += n + 1;

            Some(FdbMappedValue {
                _f: self.f.clone(),
                mapped_keyvalue,
            })
        } else {
            self.pos = self.len;
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = (self.len - self.pos) as usize;
        (rem, Some(rem))
    }
}
impl ExactSizeIterator for FdbMappedValuesIter {
    #[inline]
    fn len(&self) -> usize {
        (self.len - self.pos) as usize
    }
}
impl DoubleEndedIterator for FdbMappedValuesIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.nth_back(0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n < self.len - self.pos {
            // safe because len < original len
            self.len -= n + 1;
            let mapped_keyvalue = unsafe { self.ptr.add(self.len).read() };

            Some(FdbMappedValue {
                _f: self.f.clone(),
                mapped_keyvalue,
            })
        } else {
            self.pos = self.len;
            None
        }
    }
}
impl Drop for FdbMappedValuesIter {
    fn drop(&mut self) {
        unsafe { Vec::from_raw_parts(self.ptr as *mut FdbMappedKeyValue, 0, self.cap) };
    }
}
