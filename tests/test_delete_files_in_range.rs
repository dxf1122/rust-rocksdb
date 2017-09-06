// Copyright 2017 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use rocksdb::*;
use tempdir::TempDir;
use std::{thread, time};

#[test]
fn test_delete_files_in_range_with_iter() {
    let path = TempDir::new("_rust_rocksdb_test_delete_files_in_range_with_iter").expect("");
    let path_str = path.path().to_str().unwrap();
    let mut opts = DBOptions::new();
    opts.create_if_missing(true);
    let mut cf_opts = ColumnFamilyOptions::new();

    // DeleteFilesInRange ignore sst files in level 0.
    cf_opts.set_level_zero_file_num_compaction_trigger(1);
    let db = DB::open_cf(opts, path_str, vec!["default"], vec![cf_opts]).unwrap();
    for i in 0..3 {
        let k = format!("key{}", i);
        let v = format!("value{}", i);
        db.put(k.as_bytes(), v.as_bytes()).unwrap();
        assert_eq!(v.as_bytes(), &*db.get(k.as_bytes()).unwrap().unwrap());
    }
    // sst1 [0, 3)
    db.flush(true).unwrap();

    for i in 3..6 {
        let k = format!("key{}", i);
        let v = format!("value{}", i);
        db.put(k.as_bytes(), v.as_bytes()).unwrap();
        assert_eq!(v.as_bytes(), &*db.get(k.as_bytes()).unwrap().unwrap());
    }
    // sst2 [3, 6)
    db.flush(true).unwrap();

    for i in 6..9 {
        let k = format!("key{}", i);
        let v = format!("value{}", i);
        db.put(k.as_bytes(), v.as_bytes()).unwrap();
        assert_eq!(v.as_bytes(), &*db.get(k.as_bytes()).unwrap().unwrap());
    }
    // sst2 [6, 9)
    db.flush(true).unwrap();

    // construct iterator before DeleteFilesInRange
    let mut iter = db.iter();
    assert!(iter.seek(SeekKey::Start));

    // delete sst2
    db.delete_file_in_range(b"key2", b"key7").unwrap();
    thread::sleep(time::Duration::from_secs(1));

    let mut count = 0;
    while iter.valid() {
        iter.next();
        count = count + 1;
    }

    // iterator will pin all sst files.
    assert_eq!(count, 9);
}

#[test]
fn test_delete_files_in_range_with_snap() {
    let path = TempDir::new("_rust_rocksdb_test_delete_files_in_range_with_snap").expect("");
    let path_str = path.path().to_str().unwrap();
    let mut opts = DBOptions::new();
    opts.create_if_missing(true);
    let mut cf_opts = ColumnFamilyOptions::new();

    // DeleteFilesInRange ignore sst files in level 0.
    cf_opts.set_level_zero_file_num_compaction_trigger(1);
    let db = DB::open_cf(opts, path_str, vec!["default"], vec![cf_opts]).unwrap();
    for i in 0..3 {
        let k = format!("key{}", i);
        let v = format!("value{}", i);
        db.put(k.as_bytes(), v.as_bytes()).unwrap();
        assert_eq!(v.as_bytes(), &*db.get(k.as_bytes()).unwrap().unwrap());
    }
    // sst1 [0, 3)
    db.flush(true).unwrap();

    for i in 3..6 {
        let k = format!("key{}", i);
        let v = format!("value{}", i);
        db.put(k.as_bytes(), v.as_bytes()).unwrap();
        assert_eq!(v.as_bytes(), &*db.get(k.as_bytes()).unwrap().unwrap());
    }
    // sst2 [3, 6)
    db.flush(true).unwrap();

    for i in 6..9 {
        let k = format!("key{}", i);
        let v = format!("value{}", i);
        db.put(k.as_bytes(), v.as_bytes()).unwrap();
        assert_eq!(v.as_bytes(), &*db.get(k.as_bytes()).unwrap().unwrap());
    }
    // sst3 [6, 9)
    db.flush(true).unwrap();

    // construct snapshot before DeleteFilesInRange
    let snap = db.snapshot();

    // delete sst2
    db.delete_file_in_range(b"key2", b"key7").unwrap();
    thread::sleep(time::Duration::from_secs(1));

    let mut iter = snap.iter();
    assert!(iter.seek(SeekKey::Start));

    let mut count = 0;
    while iter.valid() {
        iter.next();
        count = count + 1;
    }

    // sst2 has been dropped.
    assert_eq!(count, 6);
}