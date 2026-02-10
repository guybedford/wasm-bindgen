#![allow(unused_imports)]
#![allow(clippy::all)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = FileSystemDirectoryReader , typescript_type = "FileSystemDirectoryReader")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `FileSystemDirectoryReader` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemDirectoryReader)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `FileSystemDirectoryReader`*"]
    pub type FileSystemDirectoryReader;
    # [wasm_bindgen (catch , method , structural , js_class = "FileSystemDirectoryReader" , js_name = readEntries)]
    #[doc = "The `readEntries()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemDirectoryReader/readEntries)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `FileSystemDirectoryReader`*"]
    pub fn read_entries_with_callback(
        this: &FileSystemDirectoryReader,
        success_callback: &::js_sys::Function,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "FileSystemDirectoryReader" , js_name = readEntries)]
    #[doc = "The `readEntries()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemDirectoryReader/readEntries)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `FileSystemDirectoryReader`*"]
    pub fn read_entries_with_callback_and_callback(
        this: &FileSystemDirectoryReader,
        success_callback: &::js_sys::Function,
        error_callback: &::js_sys::Function,
    ) -> Result<(), JsValue>;
}
