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

pub use navigator::Navigator;
pub use root::Root;
pub use accounts::Accounts;
pub use account::Account;

use crate::gui::App;
use crate::gui::platform::PlatformCallbacks;

mod navigator;
mod root;
mod accounts;
mod account;

// pub trait TitlePanelActions {
//     fn left(&self) -> Option<PanelAction>;
//     fn right(&self) -> Option<PanelAction>;
// }

#[derive(Ord, Eq, PartialOrd, PartialEq)]
pub enum ScreenId {
    Root,
    Accounts,
    Account
}

pub trait Screen {
    fn id(&self) -> ScreenId;
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        navigator: Option<&mut Navigator>,
        cb: &dyn PlatformCallbacks
    );
}
