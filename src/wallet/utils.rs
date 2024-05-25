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

use sha2::{Sha256, Digest};

/// Wallet utilities functions.
pub struct WalletUtils {}

impl WalletUtils {
    /// Setup entropy data checksum.
    pub fn setup_checksum(data: &mut Vec<u8>) {
        let mut hasher = Sha256::new();
        hasher.update(data.clone());
        let checksum = hasher.finalize();
        println!("BEFORE data: {}, checksum: {}", data.len(), checksum.len());
        data.extend(checksum);
        println!("AFTER data: {}", data.len());
    }
}