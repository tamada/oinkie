# 🛠️ The lifters

Each directory here holds what one backend needs: the script `oinkie lift`
compiles into the binary, and a README describing that backend's contract.

- [ghidra](ghidra/README.md) — P-Code, via `analyzeHeadless` and a Java script
- [binaryninja](binaryninja/README.md) — LLIL, MLIL and HLIL, via `bnpython3`
  and a Python script

## 🧪 What CI can and cannot check

Ghidra is open source and installs on a runner, so its lifter is tested end to
end on every push: `lift` really runs, and the tests in `tests/cli_test.rs`
read what it wrote.

**Binary Ninja and IDA Pro are licensed and cannot be installed on a runner.**
The call that starts them is therefore unreachable in CI, and their coverage
will not be complete. That is accepted rather than worked around: a mock of a
decompiler would assert that oinkie agrees with the mock.

What stands in for it:

| | |
| --- | --- |
| the lifting script | parsed on every push — `javac` for Java, `py_compile` for Python |
| the reader | fixtures under `testdata/`, lifted on a machine that has the tool and committed |
| the command line | built by a named function so its arguments can be asserted without the tool |
| the lift itself | run locally, by hand, before the change is proposed |

A fixture carries a repository-relative `path`, edited after lifting, because
a fresh lift records the absolute one it canonicalised.
