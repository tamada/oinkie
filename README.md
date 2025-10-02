# sniffir

Detecting the software theft, the birthmark toolkit for LLVM Bitcode.

## :runner: Usage

```sh
Usage: sniffir [OPTIONS] <COMMAND>
COMMAND
    compare      compare the given birthmarks.
    extract      extract the birthmarks from the given bitcodes.
    info         print the information of oinkie.
    exec         execute the given WASM script to perform user defined routine.
    run          extract and compare the birthmarks.
```

## Components

### Info

`Info` component prints information of oinkie.

#### Demo of `Info`

```
### Extractor

```sh
Usage: sniffir extract [OPTIONS] <BCs...>
OPTIONS
    -b, --birthmark <TYPE>    specify the birthmark type.
    -o, --output <FILE>       specify the destination. default or '-' means stdout.
```

### Comparator

### Executor

