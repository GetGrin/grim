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

/// Integrated node error type.
#[derive(Clone)]
pub enum NodeError {
    /// Storage issue.
    Storage,
    /// P2P server issue.
    P2P,
    /// API server issue.
    API,
    /// Configuration issue.
    Configuration,
    /// Unknown error.
    Unknown
}
