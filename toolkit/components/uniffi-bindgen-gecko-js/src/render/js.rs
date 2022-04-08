/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::shared::*;
use askama::Template;
use extend::ext;
use heck::{CamelCase, MixedCase, ShoutySnakeCase};
use uniffi_bindgen::interface::{
    Argument, ComponentInterface, Error, FFIFunction, Field, Function, Record, Type, Enum, Object, Constructor, Method, Literal, Radix
};

fn arg_names(args: &[&Argument]) -> String {
    args.iter()
        .map(|arg| {
            if let Some(default_value) = arg.default_value() {
                format!("{} = {}", arg.nm(), default_value.render())
            } else {
                arg.nm()
            }
        })
        .collect::<Vec<String>>()
        .join(",")
}


fn render_enum_literal(typ: &Type, variant_name: &str) -> String {
    if let Type::Enum(enum_name) = typ {
        // TODO: This does not support complex enum literals yet.
        return format!("{}.{}", enum_name.to_camel_case(), variant_name.to_shouty_snake_case());
    } else {
        panic!("Rendering an enum literal on a type that is not an enum")
    }
}
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

#[ext(name=LiteralJSExt)]
pub impl Literal {
    fn render(&self) -> String {
        match self {
            Literal::Boolean(inner) => inner.to_string(),
            Literal::String(inner) => format!("\"{}\"", inner),
            Literal::UInt(num, radix, _) =>  format!("{}", radix.render_num(num)),
            Literal::Int(num, radix, _) => format!("{}", radix.render_num(num)),
            Literal::Float(num, _) => num.clone(),
            Literal::Enum(name, typ) => render_enum_literal(typ, name),
            Literal::EmptyMap => "{}".to_string(),
            Literal::EmptySequence => "[]".to_string(),
            Literal::Null => "null".to_string(),
        }
    }
}

#[ext(name=RadixJSExt)]
pub impl Radix {
    fn render_num(&self, num: impl std::fmt::Display + std::fmt::LowerHex + std::fmt::Octal) -> String {
        match self {
            Radix::Decimal => format!("{}", num),
            Radix::Hexadecimal => format!("{:#x}", num),
            Radix::Octal => format!("{:#o}", num)
        }
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
            .map(|field| {
                if let Some(default_value) = field.default_value() {
                    format!("{} = {}", field.nm(), default_value.render())
                } else {
                    field.nm()
                }
            })
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
        arg_names(self.arguments().as_slice())
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

#[ext(name=ObjectJSExt)]
pub impl Object {
    fn nm(&self) -> String {
        self.name().to_camel_case()
    }
}

#[ext(name=ConstructorJSExt)]
pub impl Constructor {
    fn arg_names(&self) -> String {
        arg_names(&self.arguments().as_slice())

    }
}

#[ext(name=MethodJSExt)]
pub impl Method {
    fn arg_names(&self) -> String {
        arg_names(self.arguments().as_slice())
    }

    fn ffi_return_type(&self) -> String {
        self.return_type()
            .map(|t| t.ffi_converter())
            .unwrap_or("".to_string())
    }

    fn nm(&self) -> String {
        self.name().to_mixed_case()
    }
}