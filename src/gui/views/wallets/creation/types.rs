// Copyright 2023 The Grim Developers
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

/// Wallet creation step.
#[derive(PartialEq, Clone)]
pub enum Step {
    /// Mnemonic phrase input.
    EnterMnemonic,
    /// Mnemonic phrase confirmation.
    ConfirmMnemonic,
    /// Wallet connection setup.
    SetupConnection
}

impl Step {
    /// Short name representing creation step.
    pub fn name(&self) -> String {
        match *self {
            Step::EnterMnemonic => "enter_phrase".to_owned(),
            Step::ConfirmMnemonic => "confirm_phrase".to_owned(),
            Step::SetupConnection => "setup_conn".to_owned(),
        }
    }
}