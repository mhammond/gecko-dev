/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::shared::*;
use askama::Template;
use extend::ext;
use heck::{CamelCase, MixedCase, SnakeCase};
use uniffi_bindgen::interface::{ComponentInterface, FFIArgument, FFIFunction, FFIType};

#[derive(Template)]
#[template(path = "Scaffolding.cpp", escape = "none")]
pub struct CPPScaffoldingTemplate {
    pub ci: ComponentInterface,
}

#[derive(Template)]
#[template(path = "Scaffolding.h", escape = "none")]
pub struct CPPHeaderScaffoldingTemplate {
    pub ci: ComponentInterface,
}

// Define extension traits with methods used in our template code

#[ext(name=ComponentInterfaceCppExt)]
pub impl ComponentInterface {
    // Scaffolding implementation class for our WebIDL code
    fn scaffolding_class(&self) -> String {
        format!("{}Scaffolding", self.namespace().to_camel_case())
    }

    // C++ namespace we create for this scaffolding code
    fn cpp_namespace(&self) -> String {
        format!("uniffi::{}", self.namespace().to_snake_case())
    }
}

#[ext(name=FFIFunctionCppExt)]
pub impl FFIFunction {
    fn nm(&self) -> String {
        self.name().to_camel_case()
    }

    // C++ namespace we create for the scaffolding code for this function (child of
    // ComponentInterface::namespace())
    fn cpp_namespace(&self) -> String {
        self.name().to_snake_case()
    }

    // Render our arguments as a comma-separated list, where each part is what gets passed to our
    // scaffolding function by the WebIDL code generator.
    fn input_arg_list(&self) -> String {
        self.arguments()
            .into_iter()
            .map(|arg| format!("const {}& {}", arg.js_webidl_type(), arg.nm()))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn rust_name(&self) -> String {
        self.name().to_string()
    }

    fn rust_return_type(&self) -> String {
        match self.return_type() {
            Some(t) => t.rust_type(),
            None => "void".to_owned(),
        }
    }

    fn rust_arg_list(&self) -> String {
        let mut parts: Vec<String> = self.arguments().iter().map(|a| a.rust_type()).collect();
        parts.push("RustCallStatus*".to_owned());
        parts.join(", ")
    }

    fn has_args(&self) -> bool {
       !self.arguments().is_empty()
    }

    fn has_return_type(&self) -> bool {
        self.return_type().is_some()
    }
}

#[ext(name=FFITypeCppExt)]
pub impl FFIType {
    // Type for the WebIDL implementation method
    //
    // This is what we get passed from the JS code via the WebIDL bindings helper code
    fn js_webidl_type(&self) -> String {
        match self {
            FFIType::UInt8 => "uint8_t",
            FFIType::Int8 => "int8_t",
            FFIType::UInt16 => "uint16_t",
            FFIType::Int16 => "int16_t",
            FFIType::UInt32 => "uint32_t",
            FFIType::Int32 => "int32_t",
            FFIType::UInt64 => "uint64_t",
            FFIType::Int64 => "int64_t",
            FFIType::Float32 => "float",
            FFIType::Float64 => "double",
            // Pointers are handled with the "private value" API, see Scaffolding.cpp for details
            FFIType::RustArcPtr => "JS::Handle<JS::Value>",
            // The JS wrapper code uses `ArrayBuffer` since it has a nice JS API.  We input
            // `ArrayBuffer` in the C++ code, then convert it to `RustBuffer` before passing to
            // Rust.
            FFIType::RustBuffer => "ArrayBuffer",
            FFIType::ForeignBytes => unimplemented!("ForeignBytes not supported"),
            FFIType::ForeignCallback => unimplemented!("ForeignCallback not supported"),
        }
        .to_owned()
    }

    // Type for the `Args` struct
    //
    // We convert js_webidl_type -> args_type in `PrepareArgs()` in `Scaffolding.cpp` in order to:
    //
    //   - Collect successfully converted arguments from JS.  For example, we store ArrayBuffer` args in an
    //     `OwnedRustBuffer` which handles freeing them if other args fail to convert.
    //   - Avoid dereferencing JS data inside async calls.  This is important because the GC might free up the data
    //     before the worker thread processes it.  `PrepareArgs()` runs synchronously and extracts the data to pass to
    //     the worker thread.
    fn args_type(&self) -> String {
        match self {
            FFIType::RustBuffer => "OwnedRustBuffer".to_owned(),
            FFIType::RustArcPtr => "void *".to_owned(),
            _ => self.js_webidl_type(),
        }
    }

    // Type for the Rust scaffolding code
    //
    // We convert args_type -> rust_type in `Invoke()` in `Scaffolding.cpp`
    fn rust_type(&self) -> String {
        match self {
            FFIType::RustBuffer => "RustBuffer".to_owned(),
            FFIType::RustArcPtr => "void *".to_owned(),
            _ => self.js_webidl_type(),
        }
    }
}

#[ext(name=FFIArgumentCppExt)]
pub impl FFIArgument {
    fn nm(&self) -> String {
        self.name().to_mixed_case()
    }

    fn js_webidl_type(&self) -> String {
        self.type_().js_webidl_type()
    }

    fn args_type(&self) -> String {
        self.type_().args_type()
    }

    fn rust_type(&self) -> String {
        self.type_().rust_type()
    }
}
