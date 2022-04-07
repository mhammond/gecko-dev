/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::shared::*;
use askama::Template;
use extend::ext;
use heck::{CamelCase, MixedCase, ShoutySnakeCase};
use uniffi_bindgen::interface::{
    Argument, ComponentInterface, Error, FFIFunction, Field, Function, Record, Type, Enum
};

#[derive(Template)]
#[template(path = "js/wrapper.jsm", escape = "none")]
pub struct JSBindingsTemplate {
    pub ci: ComponentInterface,
}

// Define extension traits with methods used in our template code

#[ext(name=ComponentInterfaceJSExt)]
pub impl ComponentInterface {
    // Global Scaffolding object created by the WebIDL code generator
    fn scaffolding_name(&self) -> String {
        format!("{}Scaffolding", self.namespace().to_camel_case())
    }
}

#[ext(name=FFIFunctionJSExt)]
pub impl FFIFunction {
    fn nm(&self) -> String {
        self.name().to_mixed_case()
    }
}

#[ext(name=RecordJSExt)]
pub impl Record {
    fn nm(&self) -> String {
        self.name().to_camel_case()
    }

    fn constructor_field_list(&self) -> String {
        self.fields()
            .iter()
            .map(|f| f.nm())
            .collect::<Vec<String>>()
            .join(",")
    }
}

#[ext(name=FieldJSExt)]
pub impl Field {
    fn nm(&self) -> String {
        self.name().to_mixed_case()
    }

    fn write_datastream_fn(&self) -> String {
        self.type_().write_datastream_fn()
    }

    fn read_datastream_fn(&self) -> String {
        self.type_().read_datastream_fn()
    }

    fn ffi_converter(&self) -> String {
        self.type_().ffi_converter()
    }
}

#[ext(name=ArgumentJSExt)]
pub impl Argument {
    fn lower_fn_name(&self) -> String {
        format!("{}.lower", self.type_().ffi_converter())
    }

    fn nm(&self) -> String {
        self.name().to_mixed_case()
    }
}

#[ext(name=TypeJSExt)]
pub impl Type {
    fn nm(&self) -> String {
        match self {
            Type::Float64 => "Double".to_string(),
            Type::Optional(inner) => format!("Optional{}", inner.nm()),
            Type::Record(name) => name.to_camel_case(),
            Type::String => "String".to_string(),
            _ => todo!(),
        }
    }

    // Render an expression to check if two instances of this type are equal
    fn equals(&self, first: &str, second: &str) -> String {
        match self {
            Type::Record(_) => format!("{}.equals({})", first, second),
            _ => format!("{} == {}", first, second),
        }
    }

    fn write_datastream_fn(&self) -> String {
        format!("{}.write", self.ffi_converter())
    }

    fn read_datastream_fn(&self) -> String {
        format!("{}.read", self.ffi_converter())
    }

    fn ffi_converter(&self) -> String {
        format!("FfiConverter{}", self.canonical_name().to_camel_case())
    }
}

#[ext(name=EnumJSExt)]
pub impl Enum {
    fn nm(&self) -> String {
        self.name().to_camel_case()
    }
}

#[ext(name=FunctionJSExt)]
pub impl Function {
    fn arg_names(&self) -> String {
        let mut args = String::new();
        for (i, arg) in self.arguments().iter().enumerate() {
            args.push_str(&arg.name());
            if i != self.arguments().len() - 1 {
                args.push_str(",")
            }
        }
        args
    }

    fn nm(&self) -> String {
        self.name().to_mixed_case()
    }

    fn ffi_return_type(&self) -> String {
        self.return_type()
            .map(|t| t.ffi_converter())
            .unwrap_or("".to_string())
    }
}

#[ext(name=ErrorJSExt)]
pub impl Error {
    fn nm(&self) -> String {
        self.name().to_camel_case()
    }
}
