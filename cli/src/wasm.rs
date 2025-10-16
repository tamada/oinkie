use std::io::Read;
use std::path::PathBuf;

use crate::ExecuteOpts;

pub(super) fn execute(opts: ExecuteOpts) -> oinkie::Result<()> {
    let script = if opts.script == PathBuf::from("-") {
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            return Err(oinkie::OinkieError::Io(e));
        }
        Ok(buf)
    } else {
        std::fs::read_to_string(opts.script)
    };
    match script {
        Ok(script) => execute_impl(script),
        Err(e) => Err(oinkie::OinkieError::Io(e)),
    }
}

fn execute_impl(_script: String) -> oinkie::Result<()> {
    Ok(())
}
