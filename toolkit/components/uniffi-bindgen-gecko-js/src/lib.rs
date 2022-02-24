use anyhow::{bail, Result};
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::path::Path;
use uniffi_bindgen::{
    generate_external_bindings, BindingGenerator, ComponentInterface, EmptyBindingGeneratorConfig,
};

mod interface_ext;
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

struct GeckoJsBindingGenerator {
    mode: Mode,
}

// Use EmptyBindingGeneratorConfig for now
type Config = EmptyBindingGeneratorConfig;

impl GeckoJsBindingGenerator {
    fn new(mode: Mode) -> Self {
        Self { mode }
    }
}

impl BindingGenerator for GeckoJsBindingGenerator {
    type Config = Config;

    fn write_bindings(
        &self,
        ci: ComponentInterface,
        config: Self::Config,
        out_dir: &Path,
    ) -> anyhow::Result<()> {
        render::Renderer::render_file(self.mode, ci, config, out_dir)
    }
}

pub fn run_main() -> Result<()> {
    let matches = clap::App::new("uniffi-bindgen-gecko-js")
        .about("Scaffolding and bindings generator for Rust")
        .version(clap::crate_version!())
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
    let binding_generator =
        GeckoJsBindingGenerator::new(matches.value_of("mode").unwrap().try_into()?);
    generate_external_bindings(
        binding_generator,
        matches.value_of_os("udl_file").unwrap(), // Required
        matches.value_of_os("config"),
        matches.value_of_os("out_dir"),
    )
}
