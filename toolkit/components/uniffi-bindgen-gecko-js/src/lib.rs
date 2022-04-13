/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Context, Result};
use heck::CamelCase;
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use uniffi_bindgen::{
    generate_external_bindings, BindingGenerator, ComponentInterface, EmptyBindingGeneratorConfig,
};

mod render;

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
enum Mode {
    WebIdl,
    CPPHeader,
    CPP,
    JS,
}

impl TryFrom<&str> for Mode {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value.to_lowercase().as_ref() {
            "webidl" => Ok(Self::WebIdl),
            "cpp-header" => Ok(Self::CPPHeader),
            "cpp" => Ok(Self::CPP),
            "js" => Ok(Self::JS),
            _ => bail!("Invalid bindings mode: {}", value),
        }
    }
}

struct GeckoJsBindingGenerator<'a> {
    mode: Mode,
    args: clap::ArgMatches<'a>,
}

// Use EmptyBindingGeneratorConfig for now
type Config = EmptyBindingGeneratorConfig;

impl<'a> GeckoJsBindingGenerator<'a> {
    fn new(mode: Mode, args: clap::ArgMatches<'a>) -> Self {
        Self { mode, args }
    }

    fn create_writer(
        &self,
        ci: &ComponentInterface,
        out_dir: &Path,
    ) -> anyhow::Result<Box<dyn Write>> {
        if self.args.is_present("stdout") {
            return Ok(Box::new(std::io::stdout()));
        }

        let out_path = if self.args.is_present("out") {
            PathBuf::from(self.args.value_of("out").unwrap())
        } else {
            let filename = self.calc_filename(ci);
            out_dir.join(&filename)
        };
        Ok(Box::new(File::create(&out_path).context(format!(
            "Failed to create {:?}",
            out_path.file_name()
        ))?))
    }

    fn calc_filename(&self, ci: &ComponentInterface) -> String {
        match self.mode {
            Mode::WebIdl => format!("{}Scaffolding.webidl", ci.namespace().to_camel_case()),
            Mode::CPP => format!("{}Scaffolding.cpp", ci.namespace().to_camel_case()),
            Mode::CPPHeader => format!("{}Scaffolding.h", ci.namespace().to_camel_case()),
            Mode::JS => format!("{}.jsm", ci.namespace().to_camel_case()),
        }
    }
}

impl<'a> BindingGenerator for GeckoJsBindingGenerator<'a> {
    type Config = Config;

    fn write_bindings(
        &self,
        ci: ComponentInterface,
        config: Self::Config,
        out_dir: &Path,
    ) -> anyhow::Result<()> {
        let writer = self.create_writer(&ci, out_dir)?;
        render::render_file(self.mode, ci, config, writer)
    }
}

pub fn run_main() -> Result<()> {
    let matches = clap::App::new("uniffi-bindgen-gecko-js")
        .about("Scaffolding and bindings generator for Rust")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("stdout")
                .long("--stdout")
                .help("Write output to STDOUT"),
        )
        .arg(
            clap::Arg::with_name("out")
                .long("--out")
                .takes_value(true)
                .help("Write output to path"),
        )
        .arg(
            clap::Arg::with_name("out_dir")
                .long("--out-dir")
                .short("-o")
                .takes_value(true)
                .help("Directory in which to write generated files. Default is same folder as .udl file."),
        )
        .arg(clap::Arg::with_name("mode").required(true))
        .arg(clap::Arg::with_name("udl_file").required(true))
        .arg(
            clap::Arg::with_name("config")
            .long("--config-path")
            .takes_value(true)
            .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess it from the UDL's file location.")
        )
        .get_matches();
    let binding_generator = GeckoJsBindingGenerator::new(
        matches.value_of("mode").unwrap().try_into()?,
        matches.clone(),
    );
    generate_external_bindings(
        binding_generator,
        matches.value_of_os("udl_file").unwrap(), // Required
        matches.value_of_os("config"),
        matches.value_of_os("out_dir"),
    )
}
