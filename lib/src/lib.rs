pub mod birthmarks;
pub mod comparators;
pub mod extractors;

pub type Result<T> = std::result::Result<T, OinkieError>;

#[derive(Debug)]
pub enum OinkieError {
    Array(Vec<OinkieError>),
    Format(String),
    Fatal(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    NoExtension(String),
    NotFile(std::path::PathBuf),
    NotFound(std::path::PathBuf),
    UnsupportedFormat(String),
    NotImplementedYet(String),
}

impl OinkieError {
    pub fn error_or<T>(item: T, err: Vec<OinkieError>) -> Result<T> {
        if err.is_empty() {
            Ok(item)
        } else if err.len() == 1 {
            Err(err.into_iter().next().unwrap())
        } else {
            Err(OinkieError::Array(err))
        }
    }

    pub fn vec_result_to_result_vec<T>(items: Vec<Result<T>>) -> Result<Vec<T>> {
        let mut res = vec![];
        let mut errs = vec![];
        for item in items {
            match item {
                Ok(i) => res.push(i),
                Err(e) => errs.push(e),
            }
        }
        OinkieError::error_or(res, errs)
    }
}