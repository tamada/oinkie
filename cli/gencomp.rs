// cli.rs is compiled into this binary as well as into oinkie, but only its
// clap derives are used here; the accessors are exercised by the oinkie
// binary, which still lints them.
#![allow(dead_code)]

mod cli;
mod info;

use clap::{Command, CommandFactory};
use clap_complete::Shell;
use std::fs::File;
use std::path::Path;

fn generate_impl(s: Shell, app: &mut Command, outdir: &Path, file: String) {
    let destfile = outdir.join(format!("{s}")).join(file);
    std::fs::create_dir_all(destfile.parent().unwrap()).unwrap();
    let bin_name = app.get_name().to_string();
    if let Ok(mut dest) = File::create(destfile) {
        clap_complete::generate(s, app, bin_name, &mut dest);
    }
}

pub fn generate(appname: &str, outdir: &Path) {
    use Shell::{Bash, Elvish, Fish, PowerShell, Zsh};

    let mut app = crate::cli::OinkieOpts::command();
    app.set_bin_name(appname);

    generate_impl(Bash, &mut app, outdir, appname.to_string());
    generate_impl(Elvish, &mut app, outdir, format!("{appname}.elv"));
    generate_impl(Fish, &mut app, outdir, format!("{appname}.fish"));
    generate_impl(PowerShell, &mut app, outdir, format!("{appname}.ps1"));
    generate_impl(Zsh, &mut app, outdir, format!("_{appname}"));
}

fn main() {
    generate("oinkie", Path::new("assets/completions"))
}
