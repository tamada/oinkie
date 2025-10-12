# oinkie command line interface

This project is an command line interface of [`oinkie`](https://github.com/tamada/onkie).


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
Usage: oinkie extract [OPTIONS] <BCs...>
OPTIONS
    -b, --birthmark <TYPE>    specify the birthmark type.
    -o, --output <FILE>       specify the destination. default or '-' means stdout.
```

### Compare

### Execute

### Formats
