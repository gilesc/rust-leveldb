#![crate_name = "leveldb"]
#![comment = "A LevelDB binding in Rust."]
#![license = "MIT"]
#![crate_type = "lib"]
#![allow(dead_code)]

// #[deny(non_camel_case_types)];
// #[deny(missing_doc)];

#![feature(macro_rules)]
#![feature(globs)]

extern crate libc;

use std::ptr::{mut_null};
//use std::string::raw::std::string::raw::from_buf;
use libc::{c_char, size_t};
use std::vec::raw::from_buf;
use std::owned::{Box};
use std::mem;

use self::cleveldb::*;
use self::options::*;

mod cleveldb;

pub mod options {
    pub enum OpenOption {
        CREATE_IF_MISSING,
        ERROR_IF_EXISTS,
        PARANOID_CHECKS,
    }

    pub enum WriteOption {
        SYNC,
    }

    pub enum ReadOption {
        VERIFY_CHECKSUM,
        FILL_CACHE,
        // USE_SNAPSHOT(),
    }
}

pub type WriteBatch<'r> = &'r [(&'r [u8], &'r [u8])];

/// A database object
pub struct DB {
    db: *mut leveldb_t
}

pub type error = String;

fn to_c_open_options(options: &[OpenOption]) -> *const leveldb_options_t {
    unsafe {
        let c_options = leveldb_options_create();
        for option in options.iter() {
            match *option {
                CREATE_IF_MISSING => {
                    leveldb_options_set_create_if_missing(c_options, 1u8);
                },
                ERROR_IF_EXISTS => {
                    leveldb_options_set_error_if_exists(c_options, 1u8);
                },
                PARANOID_CHECKS => {
                    leveldb_options_set_paranoid_checks(c_options, 1u8);
                }
            }
        }
        c_options as *const leveldb_options_t
    }
}

fn to_c_write_options(options: &[WriteOption]) -> *const leveldb_writeoptions_t {
    unsafe {
        let c_options = leveldb_writeoptions_create();
        for option in options.iter() {
            match *option {
                SYNC => {
                    leveldb_writeoptions_set_sync(c_options, 1u8);
                }
            }
        }
        c_options as *const leveldb_writeoptions_t
    }
}

fn to_c_read_options(options: &[ReadOption]) -> *const leveldb_readoptions_t {
    unsafe {
        let c_options = leveldb_readoptions_create();
        for option in options.iter() {
            match *option {
                VERIFY_CHECKSUM => {
                    leveldb_readoptions_set_verify_checksums(c_options, 1u8);
                },
                FILL_CACHE => {
                    leveldb_readoptions_set_fill_cache(c_options, 1u8);
                }
            }
        }
        c_options as *const leveldb_readoptions_t
    }
}

fn to_c_str(s: &[u8]) -> (*const c_char, size_t) {
    unsafe {
        let c_str = s.to_c_str();
        let len = c_str.len();
        (c_str.unwrap(), len as size_t)
    }
} 

impl DB {
    /// Open a database connection
    pub fn open(name: &str, options: &[OpenOption]) -> Result<Box<DB>, error> {
        unsafe {
            let c_options = to_c_open_options(options);
            let mut err: *mut c_char = mut_null();
            let c_db = leveldb_open(c_options as *const leveldb_options_t,
                name.to_c_str().unwrap(),
                mem::transmute(&mut err));
            if c_db.is_null() {
                return Err(std::string::raw::from_buf(err as *const u8));
            } else {
                return Ok(box DB{
                    db: c_db
                });
            }
        }
    }

    pub fn close(&self) {
        unsafe {
            leveldb_close(self.db);
        }
    }

    pub fn put(&self, key: &[u8], value: &[u8], options: &[WriteOption]) -> Result<(), error> {
        unsafe {
            let mut c_err: *mut c_char = mut_null();
            let (c_key, c_key_len) = to_c_str(key);
            let (c_val, c_val_len) = to_c_str(value);
            leveldb_put(self.db, to_c_write_options(options),
                c_key, c_key_len,
                c_val, c_val_len,
                mem::transmute(&mut c_err));
            if !c_err.is_null() {
                return Err(std::string::raw::from_buf(c_err as *const u8));
            } else {
                return Ok(());
            }
        }
    }

    pub fn get(&self, key: &[u8], options: &[ReadOption]) -> Result<Vec<u8>, error> {
        unsafe {
            let mut c_err: *mut c_char = mut_null();
            let (c_key, c_key_len) = to_c_str(key);
            let mut c_value_len: size_t = 0;
            let c_value = leveldb_get(self.db, to_c_read_options(options),
                c_key, c_key_len,
                mem::transmute(&mut c_value_len),
                mem::transmute(&mut c_err));
            if !c_err.is_null() {
                return Err(std::string::raw::from_buf(c_err as *const u8));
            } else {
                return Ok(from_buf(c_value as *const u8, c_value_len as uint));
            }
        }
    }

    pub fn delete(&self, key: &[u8], options: &[WriteOption]) -> Result<(), error> {
        unsafe {
            let mut c_err: *mut c_char = mut_null();
            let (c_key, c_key_len) = to_c_str(key);
            leveldb_delete(self.db, to_c_write_options(options),
                c_key, c_key_len,
                mem::transmute(&mut c_err));
            if !c_err.is_null() {
                return Err(std::string::raw::from_buf(c_err as *const u8));
            } else {
                return Ok(());
            }
        }
    }

    pub fn write(&self, write_batch: WriteBatch, options: &[WriteOption]) -> Result<(), error> {
        unsafe {
            let c_write_batch = leveldb_writebatch_create();
            for &(key, value) in write_batch.iter() {
                let (c_key, c_key_len) = to_c_str(key);
                let (c_val, c_val_len) = to_c_str(value);
                leveldb_writebatch_put(c_write_batch,
                    c_key, c_key_len,
                    c_val, c_val_len);
            }
            let mut c_err: *mut c_char = mut_null();
            leveldb_write(self.db, to_c_write_options(options),
                c_write_batch, mem::transmute(&mut c_err));
            if !c_err.is_null() {
                return Err(std::string::raw::from_buf(c_err as *const u8));
            } else {
                return Ok(());
            }
        }
    }

    pub fn iter(&self, options: &[ReadOption]) -> DBIterator {
        unsafe {
            let it = leveldb_create_iterator(self.db, to_c_read_options(options));
            leveldb_iter_seek_to_first(it);
            return DBIterator{
                iter: it
            }
        }
    }
}

pub struct DBIterator {
    iter: *mut leveldb_iterator_t
}

// TODO: this causes crashes
// impl Drop for DBIterator {
//     fn drop(&mut self) {
//         unsafe {
//             leveldb_iter_destroy(self.iter);
//         }
//     }
// }

impl Iterator<(Vec<u8>, Vec<u8>)> for DBIterator {
    fn next(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        unsafe {
            if leveldb_iter_valid(self.iter as *const leveldb_iterator_t) == 0u8 {
                return None;
            } else {
                let pair = (self.key(), self.value());
                leveldb_iter_next(self.iter);
                return Some(pair);
            }
        }
    }
}

impl DBIterator {
    pub fn prev(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        unsafe {
            // TODO: this is buggy;
            leveldb_iter_prev(self.iter);
            if leveldb_iter_valid(self.iter as *const leveldb_iterator_t) == 0u8 {
                return None;
            } else {
                let pair = (self.key(), self.value());
                return Some(pair);
            }
        }
    }

    pub fn key(&self) -> Vec<u8> {
        unsafe {
            let mut c_key_len: size_t = 0;
            let c_key = leveldb_iter_key(self.iter as *const leveldb_iterator_t,
                mem::transmute(&mut c_key_len));
            from_buf(c_key as *const u8, c_key_len as uint)
        }
    }

    pub fn value(&self) -> Vec<u8> {
        unsafe {
            let mut c_val_len: size_t = 0;
            let c_val = leveldb_iter_value(self.iter as *const leveldb_iterator_t,
                mem::transmute(&mut c_val_len));
            from_buf(c_val as *const u8, c_val_len as uint)
        }
    }

    pub fn get_error(&self) -> Option<error> {
        unsafe {
            let mut c_err: *mut c_char = mut_null();
            leveldb_iter_get_error(self.iter as *const leveldb_iterator_t,
                mem::transmute(&mut c_err));
            if !c_err.is_null() {
                return Some(std::string::raw::from_buf(c_err as *const u8));
            } else {
                return None;
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        unsafe {
            return leveldb_iter_valid(self.iter as *const leveldb_iterator_t) != 0u8;
        }
    }

    pub fn seek(&mut self, key: &[u8]) {
        unsafe {
            let (c_key, c_key_len) = to_c_str(key);
            leveldb_iter_seek(self.iter, c_key, c_key_len);
        }
    }

    pub fn seek_to_first(&mut self) {
        unsafe {
            leveldb_iter_seek_to_first(self.iter);
        }
    }

    pub fn seek_to_last(&mut self) {
        unsafe {
            leveldb_iter_seek_to_last(self.iter);
        }
    }
}
