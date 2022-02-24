/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::interface_ext::*;
use crate::{Config, Mode};
use anyhow::Context;
use askama::{Result, Template};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uniffi_bindgen::interface::ComponentInterface;

pub struct Renderer;

impl Renderer {
    pub(crate) fn render_file(
        mode: Mode,
        ci: ComponentInterface,
        _config: Config,
        out_dir: &Path,
    ) -> anyhow::Result<()> {
        let filename = Self::calc_filename(mode, &ci);
        let out_path = out_dir.join(&filename);
        let mut f = File::create(&out_path).context(format!("Failed to create {:?}", filename))?;
        write!(f, "{}", Self::render_template(mode, ci)?)?;
        Ok(())
    }

    fn calc_filename(mode: Mode, ci: &ComponentInterface) -> String {
        match mode {
            Mode::WebIdl => format!("{}.webidl", ci.scaffolding_name()),
            Mode::CPP => format!("{}.cpp", ci.scaffolding_name()),
            Mode::CPPHeader => format!("{}.h", ci.scaffolding_name()),
            Mode::JS => format!("{}.jsm", ci.js_module_name()),
        }
    }

    fn render_template(mode: Mode, ci: ComponentInterface) -> Result<String> {
        match mode {
            Mode::WebIdl => WebIDLScaffoldingTemplate { ci }.render(),
            Mode::CPP => CPPScaffoldingTemplate { ci }.render(),
            Mode::CPPHeader => CPPHeaderScaffoldingTemplate { ci }.render(),
            Mode::JS => JSBindingsTemplate { ci }.render(),
        }
    }
}

#[derive(Template)]
#[template(path = "Scaffolding.webidl", escape = "none")]
struct WebIDLScaffoldingTemplate {
    ci: ComponentInterface,
}

#[derive(Template)]
#[template(path = "Scaffolding.cpp", escape = "none")]
struct CPPScaffoldingTemplate {
    ci: ComponentInterface,
}

#[derive(Template)]
#[template(path = "Scaffolding.h", escape = "none")]
struct CPPHeaderScaffoldingTemplate {
    ci: ComponentInterface,
}

#[derive(Template)]
#[template(path = "Bindings.jsm", escape = "none")]
struct JSBindingsTemplate {
    ci: ComponentInterface,
}
