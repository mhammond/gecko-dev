/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::shared::*;
use askama::Template;
use extend::ext;
use heck::{CamelCase, MixedCase};
use uniffi_bindgen::interface::{ComponentInterface, FFIArgument, FFIFunction, FFIType};

#[derive(Template)]
#[template(path = "Scaffolding.webidl", escape = "none")]
pub struct WebIDLScaffoldingTemplate {
    pub ci: ComponentInterface,
}

// Define extension traits with methods used in our template code
//
#[ext(name=ComponentInterfaceWebIDLExt)]
pub impl ComponentInterface {
    fn scaffolding_namespace(&self) -> String {
        format!("{}Scaffolding", self.namespace().to_camel_case())
    }
}

#[ext(name=FFIFunctionWebIDLExt)]
pub impl FFIFunction {
    fn nm(&self) -> String {
        self.name().to_mixed_case()
    }

    fn return_type_name(&self) -> String {
        if self.is_async() {
            "Promise<UniFFIRustCallResult>".to_string()
        } else {
            "UniFFIRustCallResult".to_string()
        }
    }
}

#[ext(name=FFITypeWebIDLExt)]
pub impl FFIType {
    fn type_name(&self) -> String {
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
}

#[ext(name=FFIArgumentWebIDLExt)]
pub impl FFIArgument {
    fn nm(&self) -> String {
        self.name().to_mixed_case()
    }
    fn type_name(&self) -> String {
        self.type_().type_name()
    }
}
