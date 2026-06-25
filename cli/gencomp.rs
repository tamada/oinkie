use std::path::Path;

#[cfg(debug_assertions)]
mod completions {
    use clap::{Command, CommandFactory};
    use clap_complete::Shell;
    use std::fs::File;
    use std::path::Path;

    fn generate_impl(s: Shell, app: &mut Command, appname: &str, outdir: &Path, file: String) {
        let destfile = outdir.join(file);
        std::fs::create_dir_all(destfile.parent().unwrap()).unwrap();
        if let Ok(mut dest) = File::create(destfile) {
            clap_complete::generate(s, app, appname, &mut dest);
        }
    }

    pub(super) fn generate(appname: &str, outdir: &Path) {
        use Shell::{Bash, Elvish, Fish, PowerShell, Zsh};

        let mut app = crate::cli::OinkieCommand::command();
        app.set_bin_name(appname);

        generate_impl(Bash,       &mut app, appname, outdir, format!("bash/{appname}"));
        generate_impl(Elvish,     &mut app, appname, outdir, format!("elvish/{appname}"));
        generate_impl(Fish,       &mut app, appname, outdir, format!("fish/{appname}"));
        generate_impl(PowerShell, &mut app, appname, outdir, format!("powershell/{appname}"));
        generate_impl(Zsh, &mut app, appname, outdir, format!("zsh/_{appname}"));
    }
}

#[allow(dead_code, unused_variables)]
pub(crate) fn generate(appname: &str, outdir: &Path) {
    #[cfg(debug_assertions)]
    completions::generate(appname, outdir);
}
