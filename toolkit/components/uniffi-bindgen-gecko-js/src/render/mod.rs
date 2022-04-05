/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{Config, Mode};
/// Rust code for rendering templates
///
/// This module contains the `askama` templates that we use to generate our source files.  For each
/// template, we define a set of extension traits on components from `uniffi::interface` to help
/// render them.
use askama::Template;
use std::io::Write;
use uniffi_bindgen::interface::ComponentInterface;

mod cpp;
mod js;
mod shared;
mod webidl;

pub(crate) fn render_file(
    mode: Mode,
    ci: ComponentInterface,
    _config: Config,
    mut writer: Box<dyn Write>,
) -> anyhow::Result<()> {
    let output_text = match mode {
        Mode::WebIdl => webidl::WebIDLScaffoldingTemplate { ci }.render()?,
        Mode::CPP => cpp::CPPScaffoldingTemplate { ci }.render()?,
        Mode::CPPHeader => cpp::CPPHeaderScaffoldingTemplate { ci }.render()?,
        Mode::JS => js::JSBindingsTemplate { ci }.render()?,
    };

    write!(writer, "{}", output_text)?;
    Ok(())
}
