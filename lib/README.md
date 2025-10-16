# `onkie`

This is an API library for `oinkie`.

## :walking: Example

### Original (C language)

```c
int parse_max(int argc, char *argv[]) {
    if (argc == 1)
        return 10;
    return atoi(argv[1]);
}
```

### LLVM IR

The following IR codes were obtained by compiling the source code, including the above code snippet, with `clang -emit-llvm -S`.

```llvmir
; Function Attrs: noinline nounwind optnone ssp uwtable(sync)
define i32 @parse_max(i32 noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store i32 %0, ptr %4, align 4
  store ptr %1, ptr %5, align 8
  %6 = load i32, ptr %4, align 4
  %7 = icmp eq i32 %6, 1
  br i1 %7, label %8, label %9

8:                                                ; preds = %2
  store i32 10, ptr %3, align 4
  br label %14

9:                                                ; preds = %2
  %10 = load ptr, ptr %5, align 8
  %11 = getelementptr inbounds ptr, ptr %10, i64 1
  %12 = load ptr, ptr %11, align 8
  %13 = call i32 @atoi(ptr noundef %12)
  store i32 %13, ptr %3, align 4
  br label %14

14:                                               ; preds = %9, %8
  %15 = load i32, ptr %3, align 4
  ret i32 %15
}
```

### Birthmarks

##### Example

Extracting the `OpSeq` birthmark object by executing `oinkie extract -t op-seq code.ll`

```json
[
  {
    "info": {
      "name": "parse_max",
      "path": "testdata/fibonacci.ll",
      "btype": "OpSeq",
      "mode": "Function"
    },
    "elements": [
      "Alloca",
      "Alloca",
      "Alloca",
      "Store",
      "Store",
      "Load",
      "ICmp",
      "CondBr",
      "Store",
      "Br",
      "Load",
      "GetElementPtr",
      "Load",
      "Call",
      "Store",
      "Br",
      "Load",
      "Ret"
    ]
  }
]
```

##### Entries

- `info`: metadata of birthmarks.
  - `name`: the birthmark name,
  - `path`: the birthmark extracted from,
  - `btype`: the birthmark type, and
  - `mode`: the extraction mode.
- `elements`: The birthmark elements.

#### Schema

JSON schema of the birthmark object is as follows.

```json
{
    "type": "array",
    "minItems": 0,
    "items": {
        "type": "object",
        "required": [ "info", "elements" ],
        "additionalProperties": false,
        "properties": {
            "info": {
                "type": "object",
                "required": [ "name", "path", "btype", "mode" ],
                "properties": {
                    "name": {
                        "type": "string"
                    },
                    "path": {
                        "type": "string"
                    },
                    "btype": {
                        "type": "string",
                        "pattern": "(OpSeq|OpFreq...)"
                    },
                    "mode": {
                        "type": "string",
                        "pattern": "(File|Function|BasicBlock)"
                    }
                }
            },
            "elements": {
                "type": "array",
                "minItems": 0,
                "items": {
                    "type": "string"
                }
            }
        }
    }
}
```

### Comparison Results

#### Example

```json
[
  {
    "btype": "OpSeq",
    "a_info": {
      "name": "testdata/src2ll/fizzbuzz_go.ll",
      "path": "testdata/src2ll/fizzbuzz_go.ll",
      "btype": "OpSeq",
      "mode": "File"
    },
    "b_info": {
      "name": "testdata/src2ll/fizzbuzz_rs.ll",
      "path": "testdata/src2ll/fizzbuzz_rs.ll",
      "btype": "OpSeq",
      "mode": "File"
    },
    "ctype": "Simpson",
    "score": 0.05047821466524974,
    "elapsed_ms": 0.041832999999999995
  },
  ...
]
```

##### Entries

- `btype`: Birthmark type,
- `a_info`: the metadata of the birthmark of comparison A side,
- `b_info`: the metadata of the birthmark of comparison B side,
- `ctype`: The comparison type,
- `score`: the resultant similarity, and
- `elapsed_ms`: shows the required time for comparison (milliseconds).

#### Schema

```json
{
  "type": "array",
  "minItems": 0,
  "items": {
    "type": "object",
    "required": ["btype", "a_info", "b_info", "ctype", "score", "elapsed_ms"],
    "additionalProperties": false,
    "properties": {
      "btype": {
        "type": "string"
      },
      "a_info": {
        "type": "object",
        "required": [ "name", "path", "btype", "mode" ],
        "properties": {
          "name": {
            "type": "string"
          },
          "path": {
            "type": "string"
          },
          "btype": {
            "type": "string",
            "pattern": "(OpSeq|OpFreq...)"
          },
          "mode": {
            "type": "string",
            "pattern": "(File|Function|BasicBlock)"
          }
        }
      },
      "b_info": {
        "type": "object",
        "required": [ "name", "path", "btype", "mode" ],
        "properties": {
          "name": {
            "type": "string"
          },
          "path": {
            "type": "string"
          },
          "btype": {
            "type": "string",
            "pattern": "(OpSeq|OpFreq...)"
          },
          "mode": {
            "type": "string",
            "pattern": "(File|Function|BasicBlock)"
          }
        }
      },
      "ctype": {
        "type": "string",
        "pattern": "(Simpson|Jaccard|Dice|Cosine|LCS|Levenshtein)"
      },
      "score": {
        "type": "float"
      },
      "elapsed_ms": {
        "type": "float"
      }
    }
  }
}
```
