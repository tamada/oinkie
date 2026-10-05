Birthmark files written by `oinkie extract` before #131 renamed the
`Elements` type to `Function`, kept to prove that files from before the rename
still load. Both use the JSON key `elements` for the list of functions.

- `hello_clang_op-seq.json` -- `op-seq` of `../lifted/pcodes/hello_clang.json`
- `udl_op-3gram-freq.json` -- `op-3gram-freq` of `../lifted/pcodes/udl.json`,
  a shape whose JSON is a list of `[kgram, count]` pairs (#59)
