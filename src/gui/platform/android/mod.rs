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

use lazy_static::lazy_static;
use std::sync::Arc;
use parking_lot::RwLock;

use jni::JNIEnv;
use jni::objects::{JByteArray, JObject, JString, JValue};
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
    fn call_java_method(&self, name: &str, sig: &str, args: &[JValue]) -> Option<jni::sys::jvalue> {
        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe {
            JObject::from_raw(self.android_app.activity_as_ptr() as jni::sys::jobject)
        };
        if let Ok(result) = env.call_method(activity, name, sig, args) {
           return Some(result.as_jni().clone());
        }
        None
    }
}

impl PlatformCallbacks for Android {
    fn show_keyboard(&self) {
        // Disable NDK soft input show call before fix for egui.
        // self.android_app.show_soft_input(false);

        self.call_java_method("showKeyboard", "()V", &[]).unwrap();
    }

    fn hide_keyboard(&self) {
        // Disable NDK soft input hide call before fix for egui.
        // self.android_app.hide_soft_input(false);

        self.call_java_method("hideKeyboard", "()V", &[]).unwrap();
    }

    fn copy_string_to_buffer(&self, data: String) {
        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let env = vm.attach_current_thread().unwrap();
        let arg_value = env.new_string(data).unwrap();
        self.call_java_method("copyText",
                              "(Ljava/lang/String;)V",
                              &[JValue::Object(&JObject::from(arg_value))]).unwrap();
    }

    fn get_string_from_buffer(&self) -> String {
        let result = self.call_java_method("pasteText", "()Ljava/lang/String;", &[]).unwrap();
        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let j_object: jni::sys::jobject = unsafe { result.l };
        let paste_data: String = unsafe {
            env.get_string(JString::from(JObject::from_raw(j_object)).as_ref()).unwrap().into()
        };
        paste_data
    }

    fn start_camera(&self) {
        // Clear image.
        let mut w_image = LAST_CAMERA_IMAGE.write();
        *w_image = None;
        // Start camera.
        self.call_java_method("startCamera", "()V", &[]).unwrap();
    }

    fn stop_camera(&self) {
        // Stop camera.
        self.call_java_method("stopCamera", "()V", &[]).unwrap();
        // Clear image.
        let mut w_image = LAST_CAMERA_IMAGE.write();
        *w_image = None;
    }

    fn camera_image(&self) -> Option<(Vec<u8>, u32)> {
        let r_image = LAST_CAMERA_IMAGE.read();
        if r_image.is_some() {
            return Some(r_image.clone().unwrap());
        }
        None
    }

    fn can_switch_camera(&self) -> bool {
        let result = self.call_java_method("camerasAmount", "()I", &[]).unwrap();
        let amount = unsafe { result.i };
        amount > 1
    }

    fn switch_camera(&self) {
        self.call_java_method("switchCamera", "()V", &[]).unwrap();
    }

    fn share_data(&self, name: String, data: Vec<u8>) -> Result<(), std::io::Error> {
        Ok(())
    }
}

lazy_static! {
    static ref LAST_CAMERA_IMAGE: Arc<RwLock<Option<(Vec<u8>, u32)>>> = Arc::new(RwLock::new(None));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Callback from Java code with last entered character from soft keyboard.
pub extern "C" fn Java_mw_gri_android_MainActivity_onCameraImage(
    env: JNIEnv,
    _class: JObject,
    buff: jni::sys::jbyteArray,
    rotation: jni::sys::jint,
) {
    let arr = unsafe { JByteArray::from_raw(buff) };
    let image : Vec<u8> = env.convert_byte_array(arr).unwrap();
    let mut w_image = LAST_CAMERA_IMAGE.write();
    *w_image = Some((image, rotation as u32));
}