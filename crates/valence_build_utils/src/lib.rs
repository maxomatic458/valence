#![doc = include_str!("../README.md")]

use std::path::Path;
use std::process::Command;
use std::{env, fs};

use anyhow::Context;
use proc_macro2::{Ident, Span, TokenStream};

pub fn write_generated_file(content: TokenStream, out_file: &str) -> anyhow::Result<()> {
    let out_dir = env::var_os("OUT_DIR").context("failed to get OUT_DIR env var")?;
    let path = Path::new(&out_dir).join(out_file);
    let code = content.to_string();

    fs::write(&path, code)?;

    // Try to format the output for debugging purposes.
    // Doesn't matter if rustfmt is unavailable.
    let _ = Command::new("rustfmt").arg(path).output();

    Ok(())
}

/// Parses a [`proc_macro2::Ident`] from a `str`. Rust keywords are prepended
/// with underscores to make them valid identifiers.
pub fn ident<I: AsRef<str>>(s: I) -> Ident {
    let s = s.as_ref().trim();

    // Parse the ident from a str. If the string is a Rust keyword, stick an
    // underscore in front.
    syn::parse_str::<Ident>(s)
        .unwrap_or_else(|_| Ident::new(format!("_{s}").as_str(), Span::call_site()))
}

#[track_caller]
pub fn rerun_if_changed<const N: usize>(files: [&str; N]) {
    for file in files {
        assert!(
            Path::new(file).exists(),
            "File \"{file}\" does not exist. Did you forget to update the path?"
        );

        println!("cargo:rerun-if-changed={file}");
    }
}
