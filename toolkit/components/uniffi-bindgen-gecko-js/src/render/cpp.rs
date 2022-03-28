/* This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use askama::Template;
use extend::ext;
use heck::{CamelCase, MixedCase, SnakeCase};
use uniffi_bindgen::interface::{ComponentInterface, FFIArgument, FFIFunction, FFIType};
use super::shared::*;

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
        self
            .arguments()
            .into_iter()
            .map(|arg| format!("const {}& {}", arg.type_name(), arg.nm()))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn rust_name(&self) -> String {
        self.name().to_snake_case()
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

    fn returns_rust_buffer(&self) -> bool {
        matches!(self.return_type(), Some(FFIType::RustBuffer))
    }
}

#[ext(name=FFITypeCppExt)]
pub impl FFIType {
    // Type for the WebIDL implementation method
    fn type_name(&self) -> String {
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
            FFIType::RustArcPtr => "uint64_t",
            // The JS wrapper code uses `ArrayBuffer` since it has a nice JS API.  We input
            // `ArrayBuffer` in the C++ code, then convert it to `RustBuffer` before passing to
            // Rust.
            FFIType::RustBuffer => "ArrayBuffer",
            FFIType::ForeignBytes => unimplemented!("ForeignBytes not supported"),
            FFIType::ForeignCallback => unimplemented!("ForeignCallback not supported"),
        }
        .to_owned()
    }

    // Type for the Rust scaffolding code
    fn rust_type(&self) -> String {
        match self {
            FFIType::RustBuffer => "RustBuffer".to_owned(),
            _ => self.type_name(),
        }
    }
}

#[ext(name=FFIArgumentCppExt)]
pub impl FFIArgument {
    fn nm(&self) -> String {
        self.name().to_mixed_case()
    }

    fn type_name(&self) -> String {
        self.type_().type_name()
    }

    fn rust_type(&self) -> String {
        self.type_().rust_type()
    }

    fn is_rust_buffer(&self) -> bool {
        matches!(self.type_(), FFIType::RustBuffer)
    }
}
