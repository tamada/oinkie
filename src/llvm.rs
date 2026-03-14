use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Op {

}

impl crate::Op for Op {
    fn mnemonic(&self) -> &str {
        unimplemented!()
    }

    fn code(&self) -> u32 {
        unimplemented!()
    }

    fn inputs(&self) -> Vec<String> {
        unimplemented!()
    }

    fn ret(&self) -> Option<String> {
        unimplemented!()
    }
}
