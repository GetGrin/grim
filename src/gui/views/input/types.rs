// Copyright 2025 The Grim Developers
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

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Software keyboard input type.
#[derive(Clone, PartialOrd, PartialEq)]
pub enum KeyboardLayout {
    TEXT, SYMBOLS, NUMBERS
}

/// Software keyboard input event.
#[derive(Clone)]
pub enum KeyboardEvent {
    TEXT(String), CLEAR, ENTER
}

/// Software keyboard Window State.
#[derive(Clone)]
pub struct KeyboardState {
    /// Last input event.
    pub last_event: Arc<Option<KeyboardEvent>>,
    /// Current layout.
    pub layout: Arc<KeyboardLayout>,
    /// Flag to enter uppercase symbol first.
    pub shift: Arc<AtomicBool>,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            last_event: Arc::new(None),
            layout: Arc::new(KeyboardLayout::TEXT),
            shift: Arc::new(AtomicBool::new(false)),
        }
    }
}