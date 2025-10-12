# :pig_nose: oinkie :pig2: 

[![Version](https://shields.io/badge/Version-0.1.0-blue)](https://github.com/tamada/oinkie/releases/tag/v0.1.0)
[![License-MIT](https://shields.io/badge/License-MIT-blue)](https://github.com/tamada/oinkie/blob/main/LICENSE)

[![Coverage Status](https://coveralls.io/repos/github/tamada/oinkie/badge.svg)](https://coveralls.io/github/tamada/oinkie)

Detecting software theft, the birthmark toolkit for LLVM IR and BC.

## :speaking_head: Overview

Software theft is difficult to detect because it is conducted stealthily, and the source code of the stolen software remains private.
Compilers and their options sensitively alter the binary formats (including executables) of software.
The problem is further complicated by the vast amount of software worldwide.
Therefore, we need a method to detect software theft targeting binary formats from large software repositories.

For this, Tamada et al. proposed the concept of software birthmarking in 2004.
It refers to the native characteristics of the programs and allows for comparison between them.
The similarities of the two birthmarks reflect how similar the original programs are.

This toolkit extracts them from the binary code and compares them to calculate the similarities between two birthmarks.
The high similarity suggests that either program is suspected of being a copy of the other.

## :runner: Usage

```sh
Usage: oinkie [OPTIONS] <COMMAND>
COMMAND
    compare      compare the given birthmarks.
    extract      extract the birthmarks from the given bitcodes.
    info         print the information of oinkie.
    exec         execute the given WASM script to perform user defined routine.
    run          extract and compare the birthmarks.
```

### Info

`Info` component prints information of oinkie. :pig2:

#### Demo of `Info`

### Extract

```sh
Usage: sniffir extract [OPTIONS] <BCs...>
OPTIONS
    -b, --birthmark <TYPE>    specify the birthmark type.
    -o, --output <FILE>       specify the destination. default or '-' means stdout.
```

### Compare

### Execute

### Formats

## :walking: Example

#### Original (C language)

```c
int parse_max(int argc, char *argv[]) {
    if (argc == 1)
        return 10;
    return atoi(argv[1]);
}
```

#### LLVM IR

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

#### Birthmarks

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

###### Entries

- `info`: metadata of birthmarks.
  - `name`: the birthmark name,
  - `path`: the birthmark extracted from,
  - `btype`: the birthmark type, and
  - `mode`: the extraction mode.
- `elements`: The birthmark elements.

##### Schema

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

#### Comparison Results

##### Example

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

###### Entries

- `btype`: Birthmark type,
- `a_info`: the metadata of the birthmark of comparison A side,
- `b_info`: the metadata of the birthmark of comparison B side,
- `ctype`: The comparison type,
- `score`: the resultant similarity, and
- `elapsed_ms`: shows the required time for comparison (milliseconds).

##### Schema

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


## :info: About

### The origin of the tool name `oinkie`

The previous version of this tool is [`pochi`](https://github.com/tamada/pochi), which is the birthmark toolkit for the JVM platform. The `pochi` is a dog that said "dig dig, here" and finds the treasures in the Japanese old tale "The old man who made flowers bloom." The tool finds clues of piracy from the binary code, as illustrated by the example of the dog above.

The purpose of `oinkie` is the same as `pochi` for the other platform, LLVM IR/BC. Hence, another tool name is wanted, such as an animal, a concept, or a famous person. From this background, I came up with an idea for a pig that finds a truffle. However, truffle is already used in [GraalVM](https://www.graalvm.org/latest/graalvm-as-a-platform/language-implementation-framework/). Then, I asked Microsoft Copilot, What is the famous name of the truffle pig? The name `oinkie` is one answer to the question.

### :jack_o_lantern: Logo



### :scroll: Academic Papers

#### By myself

1. Haruaki Tamada, Masahide Nakamura, Akito Monden, and Kenichi Matsumoto, ''Design and Evaluation of Birthmarks for Detecting Theft of Java Programs,'' Proc. IASTED International Conference on Software Engineering (IASTED SE 2004), pp. 569--575, February 2004 (Innsbruck, Austria). [![Link](https://img.shields.io/badge/Link-cir.nii.ac.jp-orange)](https://cir.nii.ac.jp/crid/1572824500637007232)
    - proposed a concept of software birthmarks, and the birthmark type of CVFV, UC, SMC, and IS.
2. Haruaki Tamada, Keiji Okamoto, Masahide Nakamura, Akito Monden, and Kenichi Matsumoto, ''Dynamic Software Birthmarks to Detect the Theft of Windows Applications,'' Proc. International Symposium on Future Software Technology 2004 (ISFST 2004), October 2004 (Xi'an, China).[![Link](https://img.shields.io/badge/Link-cir.nii.ac.jp-orange)](https://cir.nii.ac.jp/crid/1570291225759950720)
    - proposed a concept of dynamic software birthmarks, and the birthmark types of EXESEQ and EXEFREQ.
3. Haruaki Tamada, Masahide Nakamura, Akito Monden, and Ken-ichi Matsumoto, ''Java Birthmarks --Detecting the Software Theft--,'' IEICE Transactions on Information and Systems, Vol. E88-D, No. 9, pp. 2148--2158, September 2005. [![Link](https://img.shields.io/badge/Link-cir.nii.ac.jp-orange)](https://cir.nii.ac.jp/crid/1570854177800875008)
    - proposed a concept of static software birthmarks, and the birthmark types of CVFV, UC, and SMC.
4. Takehiro Tsuzaki, Teruaki Yamamoto, Haruaki Tamada, and Akito Monden, ''A Fuzzy Hashing Technique for Large Scale Software Birthmarks,'' Proc. 15th IEEE/ACIS International Conference on Computer and Information Science ([ICIS 2016](http://www.acisinternational.org/icis2016/)), pp. 867--872, July 2016 (Oakayama, Japan). [![Link](https://img.shields.io/badge/Link-ieeexplore.ieee.org-orange)](http://ieeexplore.ieee.org/document/7550868/)
    - Introduce the fuzzy hash technique for comparing birthmarks for faster comparison.
5. Jun Nakamura, and Haruaki Tamada, ''Fast Comparison of Software Birthmarks for Detecting the Theft with the Search Engine,'' Proc. of the 4th International Conference on Applied Computing & Information Technology (ACIT 2016), pp. 152--157, December 2016 (UNLV, Las Vegas, NV, USA). [![Link](https://img.shields.io/badge/Link-ieeexplore.ieee.org-orange)](http://ieeexplore.ieee.org/document/7916974/)
    - Introduce a search engine for faster comparison.
6. Jun Nakamura and Haruaki Tamada, ''mituba: Scaling up Software Theft Detection with the Search Engine,'' Proc. International Conference on Software Engineering and Information Management ([ICSIM 2018](http://www.icsim.org/icsim2018.html)), pp. 6--10, January 2018 (Casablanca, Morocco). [![Link](https://img.shields.io/badge/Link-dl.acm.org-orange)](https://dl.acm.org/citation.cfm?id=3178475)
    - Upgrading the method of ACIT 2016.
7. Nikolay Fedorov, Hiroki Inayoshi, Haruaki Tamada, Akito Monden, ''Comparison of Similarity Functions for n-gram Software Birthmarks,'' The 6th World Symposium on Software Engineering ([WSSE 2024](https://wsse.org/)), pp. 169--176, September 2024 (Kyoto, Japan). [![Link](https://img.shields.io/badge/Link-dl.acm.org-orange)](https://dl.acm.org/doi/10.1145/3698062.3698087)

#### Popular papers on software birthmarks

##### $k$-gram-based birthmarks

- Ginger Myles and Christian Collberg, "$k$-gram-based software birthmarks," In Proc. the 2005 ACM Symposium on Applied Computing, pp.314--318, March 2005. [![Link](https://img.shields.io/badge/Link-dl.acm.org-orange)](https://dl.acm.org/doi/10.1145/1066677.1066753)

##### Whole Program Path (Dynamic Software Birthmarks)

- Ginger Myles and Christian Collberg, "Detecting Software Theft via Whole Program Path Birthmarks," In Proc. International Conference on Information Security 2004, pp.404–-415, 2004. [![Link](https://img.shields.io/badge/Link-link.shpringer.com-orange)](https://link.springer.com/chapter/10.1007/978-3-540-30144-8_34)