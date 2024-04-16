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

use winit::platform::android::activity::AndroidApp;
use crate::gui::platform::PlatformCallbacks;

#[derive(Clone)]
pub struct Android {
    android_app: AndroidApp,
}

impl Android {
    pub fn new(app: AndroidApp) -> Self {
        Self {
            android_app: app,
        }
    }
}

impl PlatformCallbacks for Android {
    fn show_keyboard(&self) {
        // Disable NDK soft input show call before fix for egui.
        // self.android_app.show_soft_input(false);

        use jni::objects::{JObject};

        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe {
            JObject::from_raw(self.android_app.activity_as_ptr() as jni::sys::jobject)
        };
        let _ = env.call_method(
            activity,
            "showKeyboard",
            "()V",
            &[]
        ).unwrap();
    }

    fn hide_keyboard(&self) {
        // Disable NDK soft input hide call before fix for egui.
        // self.android_app.hide_soft_input(false);

        use jni::objects::{JObject};

        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe {
            JObject::from_raw(self.android_app.activity_as_ptr() as jni::sys::jobject)
        };
        let _ = env.call_method(
            activity,
            "hideKeyboard",
            "()V",
            &[]
        ).unwrap();
    }

    fn copy_string_to_buffer(&self, data: String) {
        use jni::objects::{JObject, JValue};

        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe {
            JObject::from_raw(self.android_app.activity_as_ptr() as jni::sys::jobject)
        };
        let arg_value = env.new_string(data).unwrap();
        let _ = env.call_method(
            activity,
            "copyText",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&JObject::from(arg_value))]
        ).unwrap();
    }

    fn get_string_from_buffer(&self) -> String {
        use jni::objects::{JObject, JString};

        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe {
            JObject::from_raw(self.android_app.activity_as_ptr() as jni::sys::jobject)
        };
        let result = env.call_method(
            activity,
            "pasteText",
            "()Ljava/lang/String;",
            &[]
        ).unwrap();
        let j_object: jni::sys::jobject = unsafe { result.as_jni().l };
        let paste_data: String = unsafe {
            env.get_string(JString::from(JObject::from_raw(j_object)).as_ref()).unwrap().into()
        };
        paste_data
    }
}