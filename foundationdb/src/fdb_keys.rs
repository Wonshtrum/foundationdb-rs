// Copyright 2022 foundationdb-rs developers, https://github.com/Clikengo/foundationdb-rs/graphs/contributors
// Copyright 2013-2018 Apple, Inc and the FoundationDB project authors.
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Definitions of FDBKeys, used in api version 700 and more.

use crate::error;
use crate::future::FdbFutureHandle;
use crate::mem::read_unaligned_slice;
use crate::{FdbError, FdbResult};
use foundationdb_sys as fdb_sys;
use std::fmt;
use std::mem::ManuallyDrop;
use std::ops::Deref;

/// An slice of keys owned by a FoundationDB future
pub struct FdbKeys {
    _f: FdbFutureHandle,
    keys: Vec<FdbKey>,
}
unsafe impl Sync for FdbKeys {}
unsafe impl Send for FdbKeys {}

impl TryFrom<FdbFutureHandle> for FdbKeys {
    type Error = FdbError;

    fn try_from(f: FdbFutureHandle) -> FdbResult<Self> {
        let mut keys = std::ptr::null();
        let mut len = 0;

        error::eval(unsafe { fdb_sys::fdb_future_get_key_array(f.as_ptr(), &mut keys, &mut len) })?;

        Ok(FdbKeys {
            _f: f,
            keys: unsafe { read_unaligned_slice(keys as *const _, len) },
        })
    }
}

impl Deref for FdbKeys {
    type Target = [FdbKey];
    fn deref(&self) -> &Self::Target {
        &self.keys
    }
}

impl AsRef<[FdbKey]> for FdbKeys {
    fn as_ref(&self) -> &[FdbKey] {
        self.deref()
    }
}

impl<'a> IntoIterator for &'a FdbKeys {
    type Item = &'a FdbKey;
    type IntoIter = std::slice::Iter<'a, FdbKey>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().iter()
    }
}

impl IntoIterator for FdbKeys {
    type Item = FdbRowKey;
    type IntoIter = FdbKeysIter;

    fn into_iter(self) -> Self::IntoIter {
        let keys = ManuallyDrop::new(self.keys);
        FdbKeysIter {
            f: std::rc::Rc::new(self._f),
            ptr: keys.as_ptr(),
            len: keys.len(),
            cap: keys.capacity(),
            pos: 0,
        }
    }
}

/// An iterator of keyvalues owned by a foundationDB future
pub struct FdbKeysIter {
    f: std::rc::Rc<FdbFutureHandle>,
    ptr: *const FdbKey,
    len: usize,
    cap: usize,
    pos: usize,
}

impl Iterator for FdbKeysIter {
    type Item = FdbRowKey;
    fn next(&mut self) -> Option<Self::Item> {
        #[allow(clippy::iter_nth_zero)]
        self.nth(0)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n < self.len - self.pos {
            // safe because pos < self.len
            let row_key = unsafe { self.ptr.add(self.pos + n).read() };
            self.pos += n + 1;

            Some(FdbRowKey {
                _f: self.f.clone(),
                row_key,
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
impl ExactSizeIterator for FdbKeysIter {
    #[inline]
    fn len(&self) -> usize {
        (self.len - self.pos) as usize
    }
}
impl DoubleEndedIterator for FdbKeysIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.nth_back(0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n < self.len - self.pos {
            // safe because len < original len
            self.len -= n + 1;
            let row_key = unsafe { self.ptr.add(self.len).read() };

            Some(FdbRowKey {
                _f: self.f.clone(),
                row_key,
            })
        } else {
            self.pos = self.len;
            None
        }
    }
}
impl Drop for FdbKeysIter {
    fn drop(&mut self) {
        unsafe { Vec::from_raw_parts(self.ptr as *mut FdbKey, 0, self.cap) };
    }
}

/// A row key you can own
///
/// Until dropped, this might prevent multiple key/values from beeing freed.
/// (i.e. the future that own the data is dropped once all data it provided is dropped)
pub struct FdbRowKey {
    _f: std::rc::Rc<FdbFutureHandle>,
    row_key: FdbKey,
}

impl Deref for FdbRowKey {
    type Target = FdbKey;
    fn deref(&self) -> &Self::Target {
        &self.row_key
    }
}
impl AsRef<FdbKey> for FdbRowKey {
    fn as_ref(&self) -> &FdbKey {
        self.deref()
    }
}
impl PartialEq for FdbRowKey {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl Eq for FdbRowKey {}
impl fmt::Debug for FdbRowKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

#[repr(transparent)]
/// An FdbKey, owned by a FoundationDB Future
pub struct FdbKey(fdb_sys::FDBKey);

impl FdbKey {
    /// retrieves the associated key
    pub fn key(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0.key as *const u8, self.0.key_length as usize) }
    }
}

impl PartialEq for FdbKey {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for FdbKey {}

impl fmt::Debug for FdbKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?})", crate::tuple::Bytes::from(self.key()),)
    }
}
