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

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use rkv::backend::{SafeMode, SafeModeDatabase, SafeModeEnvironment};
use rkv::{Manager, Rkv, SingleStore, StoreOptions, Value};

/// Transaction height storage.
pub struct TxHeightStore {
    env: Arc<RwLock<Rkv<SafeModeEnvironment>>>,
    /// Confirmed heights.
    confirmed: SingleStore<SafeModeDatabase>,
    /// Broadcasting heights.
    broadcasting: SingleStore<SafeModeDatabase>
}

impl TxHeightStore {
    /// Create new transaction height storage from provided directory.
    pub fn new(dir: PathBuf) -> Self {
        let mut manager = Manager::<SafeModeEnvironment>::singleton().write().unwrap();
        let created_arc = manager.get_or_create(dir.as_path(), Rkv::new::<SafeMode>).unwrap();
        let env = created_arc.clone();
        let k = created_arc.read().unwrap();

        let confirmed = k.open_single("tx_height", StoreOptions::create()).unwrap();
        let broadcasting = k.open_single("broadcast_tx_height", StoreOptions::create()).unwrap();
        Self {
            env,
            confirmed,
            broadcasting
        }
    }

    /// Read transaction height from database.
    pub fn read_tx_height(&self, slate_id: &String) -> Option<u64> {
        let env = self.env.read().unwrap();
        let reader = env.read().unwrap();
        if let Ok(value) = self.confirmed.get(&reader, slate_id) {
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
    pub fn write_tx_height(&self, slate_id: &String, height: u64) {
        let env = self.env.read().unwrap();
        let mut writer = env.write().unwrap();
        self.confirmed.put(&mut writer, slate_id, &Value::U64(height)).unwrap();
        writer.commit().unwrap();
    }

    /// Read broadcasting height from database.
    pub fn read_broadcasting_height(&self, slate_id: &String) -> Option<u64> {
        let env = self.env.read().unwrap();
        let reader = env.read().unwrap();
        if let Ok(value) = self.broadcasting.get(&reader, slate_id) {
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
    pub fn write_broadcasting_height(&self, slate_id: &String, height: u64) {
        let env = self.env.read().unwrap();
        let mut writer = env.write().unwrap();
        self.broadcasting.put(&mut writer, slate_id, &Value::U64(height)).unwrap();
        writer.commit().unwrap();
    }

    /// Delete broadcasting height from database.
    pub fn delete_broadcasting_height(&self, slate_id: &String) {
        let env = self.env.read().unwrap();
        let mut writer = env.write().unwrap();
        self.broadcasting.delete(&mut writer, slate_id).unwrap_or_default();
        writer.commit().unwrap();
    }
}
