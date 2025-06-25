// Copyright 2024 The Grim Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::{Arc, RwLock};
use rkv::backend::{Lmdb, LmdbDatabase, LmdbEnvironment};
use rkv::{IntegerStore, Manager, Rkv, StoreOptions, Value};

/// Transaction height storage.
pub struct TxHeightStore {
    env: Arc<RwLock<Rkv<LmdbEnvironment>>>,
    /// Confirmed heights.
    confirmed: IntegerStore<LmdbDatabase, u32>,
    /// Broadcasting heights.
    broadcasting: IntegerStore<LmdbDatabase, u32>
}

impl TxHeightStore {
    /// Create new transaction height storage from provided directory.
    pub fn new(dir: String) -> Self {
        let mut manager = Manager::<LmdbEnvironment>::singleton().write().unwrap();
        let env_arc = manager.get_or_create(std::path::Path::new(&dir), Rkv::new::<Lmdb>).unwrap();

        let env_arc_store = env_arc.clone();
        let env = env_arc_store.read().unwrap();
        let confirmed = env.open_integer("tx_height", StoreOptions::create()).unwrap();
        let broadcasting = env.open_integer("broadcast_tx_height", StoreOptions::create()).unwrap();
        Self {
            env: env_arc,
            confirmed,
            broadcasting
        }
    }

    /// Read transaction height from database.
    pub fn read_tx_height(&self, id: u32) -> Option<u64> {
        let env = self.env.read().unwrap();
        let reader = env.read().unwrap();
        if let Ok(value) = self.confirmed.get(&reader, id) {
            if let Some(height) = value {
                return match height {
                    Value::U64(v) => Some(v),
                    _ => None
                };
            }
            return None;
        }
        None
    }

    /// Write transaction height to database.
    pub fn write_tx_height(&self, id: u32, height: u64) {
        let env = self.env.read().unwrap();
        let mut writer = env.write().unwrap();
        self.confirmed.put(&mut writer, id, &Value::U64(height)).unwrap();
        writer.commit().unwrap();
    }

    /// Read broadcasting height from database.
    pub fn read_broadcasting_height(&self, id: u32) -> Option<u64> {
        let env = self.env.read().unwrap();
        let reader = env.read().unwrap();
        if let Ok(value) = self.broadcasting.get(&reader, id) {
            if let Some(height) = value {
                return match height {
                    Value::U64(v) => Some(v),
                    _ => None
                };
            }
            return None;
        }
        None
    }

    /// Write broadcasting height to database.
    pub fn write_broadcasting_height(&self, id: u32, height: u64) {
        let env = self.env.read().unwrap();
        let mut writer = env.write().unwrap();
        self.broadcasting.put(&mut writer, id, &Value::U64(height)).unwrap();
        writer.commit().unwrap();
    }

    /// Delete broadcasting height from database.
    pub fn delete_broadcasting_height(&self, id: u32) {
        let env = self.env.read().unwrap();
        let mut writer = env.write().unwrap();
        self.broadcasting.delete(&mut writer, id).unwrap_or_default();
        writer.commit().unwrap();
    }
}
