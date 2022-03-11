/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Extension traits for `UniFFI` interface types.
//!
//! This module implements Gecko-JS specific extension traits on these types, which we then use in
//! the rendering code.

use extend::ext;
use heck::{CamelCase, MixedCase, SnakeCase};
use uniffi_bindgen::interface::{ComponentInterface, FFIArgument, FFIFunction, FFIType, Record, Field, Argument, Type, Function};

// Note: The code below makes heavy use of the `[extend::ext]` attribute. The basic idea is that
// for type `Foo`, we create and implement the `FooExt` trait in one code block.  See
// https://lib.rs/crates/extend for details.

#[ext]
pub impl ComponentInterface {
    fn js_module_name(&self) -> String {
        self.namespace().to_camel_case()
    }

    fn scaffolding_name(&self) -> String {
        format!("{}Scaffolding", self.namespace().to_camel_case())
    }

    fn cpp_class_name(&self) -> String {
        format!("{}Scaffolding", self.namespace().to_camel_case())
    }

    fn cpp_namespace(&self) -> String {
        format!("uniffi::{}", self.namespace().to_snake_case())
    }
}

#[ext]
pub impl FFIFunction {
    fn is_async(&self) -> bool {
        // TODO check `uniffi.toml` or some other configuration to figure this out
        true
    }

    fn webidl_name(&self) -> String {
        self.name().to_mixed_case()
    }

    fn cpp_name(&self) -> String {
        self.name().to_camel_case()
    }

    fn cpp_namespace(&self) -> String {
        self.name().to_snake_case()
    }

    fn rust_name(&self) -> String {
        self.name().to_snake_case()
    }

    fn js_name(&self) -> String {
        self.name().to_mixed_case()
    }

    fn webidl_return_type(&self) -> String {
        if self.is_async() {
            "Promise<UniFFIRustCallResult>".to_string()
        } else {
            "UniFFIRustCallResult".to_string()
        }
    }

    fn rust_return_type(&self) -> String {
        match self.return_type() {
            Some(t) => t.rust_type(),
            None => "void".to_owned(),
        }
    }

    // Render our arguments as a comma-separated list, where each type gets passed to our
    // scaffolding function by the WebIDL code generator.
    fn cpp_input_arg_list(&self) -> String {
        self
            .arguments()
            .into_iter()
            .map(|arg| format!("const {}& {}", arg.cpp_type(), arg.cpp_name()))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn rust_arg_list(&self) -> String {
        let mut parts: Vec<String> = self.arguments().iter().map(|a| a.rust_type()).collect();
        parts.push("RustCallStatus*".to_owned());
        parts.join(", ")
    }

    // Currently needed because we handle `RustBuffer` specially in the C++ code, I wish we could
    // rework things to not need this.
    fn returns_rust_buffer(&self) -> bool {
        matches!(self.return_type(), Some(FFIType::RustBuffer))
    }
}

#[ext]
pub impl FFIType {
    fn webidl_type(&self) -> String {
        match self {
            FFIType::UInt8 => "octet",
            FFIType::Int8 => "byte",
            FFIType::UInt16 => "unsigned short",
            FFIType::Int16 => "short",
            FFIType::UInt32 => "unsigned long",
            FFIType::Int32 => "long",
            FFIType::UInt64 => "unsigned long long",
            FFIType::Int64 => "long long",
            FFIType::Float32 => "float",
            FFIType::Float64 => "double",
            FFIType::RustArcPtr => "long long",
            FFIType::RustBuffer => "ArrayBuffer",
            FFIType::ForeignBytes => unimplemented!("ForeignBytes not supported"),
            FFIType::ForeignCallback => unimplemented!("ForeignCallback not supported"),
        }
        .to_owned()
    }

    fn cpp_type(&self) -> String {
        match self {
            FFIType::UInt8 => "uint_8",
            FFIType::Int8 => "int_8",
            FFIType::UInt16 => "uint_16",
            FFIType::Int16 => "int_16",
            FFIType::UInt32 => "uint_32",
            FFIType::Int32 => "int_32",
            FFIType::UInt64 => "uint_64",
            FFIType::Int64 => "int_64",
            FFIType::Float32 => "float",
            FFIType::Float64 => "double",
            FFIType::RustArcPtr => "uint_64",
            // The JS wrapper code uses `ArrayBuffer` since it has a nice JS API.  We input
            // `ArrayBuffer` in the C++ code, then convert it to `RustBuffer` before passing to
            // Rust.
            FFIType::RustBuffer => "ArrayBuffer",
            FFIType::ForeignBytes => unimplemented!("ForeignBytes not supported"),
            FFIType::ForeignCallback => unimplemented!("ForeignCallback not supported"),
        }
        .to_owned()
    }

    fn rust_type(&self) -> String {
        match self {
            // As mentioned above, this is mostly the same as `cpp_type()`, except for `RustBuffer`
            FFIType::RustBuffer => "RustBuffer".to_owned(),
            _ => self.cpp_type(),
        }
    }
}

#[ext]
pub impl FFIArgument {
    fn webidl_name(&self) -> String {
        self.name().to_mixed_case()
    }

    fn cpp_name(&self) -> String {
        self.name().to_mixed_case()
    }

    fn webidl_type(&self) -> String {
        self.type_().webidl_type()
    }

    fn cpp_type(&self) -> String {
        self.type_().cpp_type()
    }

    fn rust_type(&self) -> String {
        self.type_().rust_type()
    }

    // Currently needed because we handle `RustBuffer` specially in the C++ code, I wish we could
    // rework things to not need this.
    fn is_rust_buffer(&self) -> bool {
        matches!(self.type_(), FFIType::RustBuffer)
    }
}

#[ext]
pub impl Record {
    fn js_name(&self) -> String {
        self.name().to_camel_case()
    }

    fn js_constructor_field_list(&self) -> String {
        self.fields().iter().map(|f| f.js_name()).collect::<Vec<String>>().join(",")
    }
}

#[ext]
pub impl Field {
    fn js_name(&self) -> String {
        self.name().to_mixed_case()
    }

    fn write_datastream_fn(&self) -> String {
        self.type_().write_datastream_fn()
    }

    fn read_datastream_fn(&self) -> String {
        self.type_().read_datastream_fn()
    }

    fn js_ffi_converter(&self) -> String {
        self.type_().js_ffi_converter()
    }
}

#[ext]
pub impl Argument {
    fn js_lower_fn_name(&self) -> String {
        format!("FfiConverter{}.lower", self.type_().js_name())
    }

    fn js_name(&self) -> String {
        self.name().to_mixed_case()
    }
}

#[ext]
pub impl Type {
    fn js_name(&self) -> String {
        match self {
            Type::Float64 => "Double".to_string(),
            Type::Optional(inner) => format!("Optional{}", inner.js_name()),
            Type::Record(name) => name.to_camel_case(),
            _ => todo!()
        }
    }

    fn write_datastream_fn(&self) -> String {
        format!("{}.write", self.js_ffi_converter())
    }

    fn read_datastream_fn(&self) -> String {
       format!("{}.read", self.js_ffi_converter())
    }

    fn js_ffi_converter(&self) -> String {
        format!("FfiConverter{}", self.js_name())
    }
}

#[ext]
pub impl Function {
    fn is_async(&self) -> bool {
        // TODO check `uniffi.toml` or some other configuration to figure this out
        true
    }

    fn js_arg_names(&self) -> String {
        let mut args = String::new();
        for (i, arg) in self.arguments().iter().enumerate() {
            args.push_str(&arg.js_name());
            if i != self.arguments().len() - 1 {
                args.push_str(",")
            }
        }
        args
    }

    fn js_name(&self) -> String  {
        self.name().to_mixed_case()
    }

    fn js_ffi_return_type(&self) -> String {
        self.return_type().map(|t| t.js_ffi_converter()).unwrap_or("".to_string())
    }
}
