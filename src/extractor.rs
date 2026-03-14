use rustc_hash::FxHashMap;

use crate::prelude::*;

pub struct Extractor<T> {
    bt: BirthmarkType,
    args: Vec<Program<T>>,
}

impl<T: crate::Op> Extractor<T> {
    pub fn new(bt: BirthmarkType, args: Vec<Program<T>>) -> Self {
        Self { bt, args }
    }

    pub fn extract(&self) -> Result<Vec<Birthmark>> {
        let result = self.args.iter()
            .map(|p| extract_birthmark_op(p, &self.bt))
            .collect::<Vec<_>>();
        Error::vec_result_to_result_vec(result)
    }
}

fn extract_birthmark_op<T: crate::Op>(p: &Program<T>, bt: &BirthmarkType) -> Result<Birthmark> {
    let elements = p.iter().map(|f| {
        let name = f.name().to_string();
        let data = match bt {
            BirthmarkType::FcFreq => Data::Freq(extract_function_calls_freq(f)),
            BirthmarkType::FcSet => Data::Set(extract_function_calls_freq(f).into_keys().collect()),
            BirthmarkType::FcSeq => Data::Seq(extract_function_calls(f)),
            BirthmarkType::OpFreq => Data::Freq(f.ops_freq()),
            BirthmarkType::OpSet => Data::Set(f.ops_freq().into_keys().collect()),
            BirthmarkType::OpSeq => Data::Seq(f.ops().map(|s| s.into()).collect()),
            BirthmarkType::OpKgramSeq(k) => Data::KgramSeq(extract_op_kgram_seq(f, *k)),
            BirthmarkType::OpKgramFreq(k) => Data::KgramFreq(extract_op_kgram_freq(f, *k)),
            BirthmarkType::OpKgramSet(k) => Data::KgramSet(extract_op_kgram_seq(f, *k)
                .into_iter().collect()),
        };
        Elements { name, data }
    }).collect::<Vec<_>>();
    Ok(Birthmark {
        name: p.name().to_string(),
        path: p.path().to_path_buf(),
        birthmark_type: bt.clone(),
        elements,
    })
}

fn extract_op_kgram_seq<T: crate::Op>(f: &Function<T>, k: usize) -> Vec<Kgram> {
    f.ops().map(|s| s.into()).collect::<Vec<_>>()
        .windows(k)
        .map(|w| Kgram::new(w.to_vec()))
        .collect()
}

fn extract_op_kgram_freq<T: crate::Op>(f: &Function<T>, k: usize) -> FxHashMap<Kgram, usize> {
    seq_to_freq(extract_op_kgram_seq(f, k).into_iter())
}

pub(crate) fn seq_to_freq<T>(seq: impl Iterator<Item = T>) -> FxHashMap<T, usize>
where
    T: std::hash::Hash + Eq,
{
    seq.into_iter()
        .fold(FxHashMap::default(), |mut acc, item| {
            *acc.entry(item).or_insert(0) += 1;
            acc
        })
}

fn extract_function_calls<T: crate::Op>(f: &Function<T>) -> Vec<String> {
    todo!()
}

fn extract_function_calls_freq<T: crate::Op>(f: &Function<T>) -> FxHashMap<String, usize> {
    extract_function_calls(f).into_iter()
        .fold(FxHashMap::default(), |mut acc, call| {
            *acc.entry(call).or_insert(0) += 1;
            acc
        })
}
