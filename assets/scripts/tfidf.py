#! /usr/bin/env python3
"""Reweights birthmarks by TF-IDF, writing them as frequency birthmarks.

    python3 assets/scripts/tfidf.py INPUT_DIR OUTPUT_DIR [--scale N]

Every birthmark (*.json) in INPUT_DIR is read, and the same file name is
written to OUTPUT_DIR with each element's count replaced by its TF-IDF weight.
The outputs are ordinary frequency birthmarks, so `oinkie compare` and every
frequency algorithm (cosine, euclidean, weighted-jaccard, tanimoto,
jensen-shannon) read them as they are.

Accepted inputs are the sequence and frequency birthmarks -- op-seq, op-freq,
fc-seq, fc-freq, op-{k}gram-seq, op-{k}gram-freq -- since a count is what TF
is. A set has no counts and is refused. A sequence is counted into a
frequency map first, so op-seq and op-freq of the same programs give the same
output, and the two may be mixed in one INPUT_DIR. Every input must be of one
family (op, fc, or op-{k}gram with one k) and one IR: an IDF counted across
families would weigh elements that cannot occur in each other's birthmarks.

The documents are the functions of every input together, since a comparison
scores function against function:

    tf(e, f) = how often e occurs in function f
    df(e)    = how many functions of all the inputs contain e
    idf(e)   = ln((1 + N) / (1 + df(e))) + 1, N the number of functions
    weight   = round(tf * idf * scale)

The weights depend on which birthmarks are in INPUT_DIR: converting a
different set gives different weights, and only birthmarks converted together
are comparable. The table is written to OUTPUT_DIR/idf.csv, outside the
*.json a comparison is run on, so that what the weights came from is kept.

A birthmark holds whole counts, so the weights are scaled and rounded. Every
frequency algorithm scores a vector the same at any scale, so the scale only
sets how much rounding costs. With idf >= 1, no weight rounds to zero, and
every function keeps the elements it had.
"""

import argparse
import csv
import json
import math
import sys
from collections import Counter
from pathlib import Path

# The birthmark type as the file writes it, mapped to its family and whether
# its data is a sequence. A k-gram type is written as {"OpKgramSeq": k}.
PLAIN_TYPES = {
    "OpSeq": ("op", True),
    "OpFreq": ("op", False),
    "FcSeq": ("fc", True),
    "FcFreq": ("fc", False),
}
KGRAM_TYPES = {"OpKgramSeq": True, "OpKgramFreq": False}
FREQ_TYPE = {"op": "OpFreq", "fc": "FcFreq"}


class Refused(Exception):
    pass


def family_of(path, birthmark_type):
    """The family of a birthmark type (`op`, `fc` or `op-{k}gram`)."""
    if isinstance(birthmark_type, str) and birthmark_type in PLAIN_TYPES:
        return PLAIN_TYPES[birthmark_type][0]
    if isinstance(birthmark_type, dict) and len(birthmark_type) == 1:
        (name, k), = birthmark_type.items()
        if name in KGRAM_TYPES:
            return f"op-{k}gram"
    raise Refused(f"{path}: {json.dumps(birthmark_type)} has no counts; "
                  "give a seq or freq birthmark")


def freq_type(family):
    """The frequency birthmark type of a family, as the file writes it."""
    if family in FREQ_TYPE:
        return FREQ_TYPE[family]
    return {"OpKgramFreq": int(family[len("op-"):-len("gram")])}


def term_frequencies(path, data):
    """One function's elements and their counts. A k-gram is a tuple, so
    that it can be a key."""
    (shape, value), = data.items()
    if shape in ("Seq", "Freq"):
        return Counter(value)
    if shape == "KgramSeq":
        return Counter(tuple(k) for k in value)
    if shape == "KgramFreq":
        return Counter({tuple(k): n for k, n in value})
    raise Refused(f"{path}: {shape} has no counts")


def read(path):
    with open(path, encoding="utf-8") as f:
        birthmark = json.load(f)
    try:
        metadata = birthmark["metadata"]
        functions = birthmark["elements"]
    except (KeyError, TypeError):
        raise Refused(f"{path}: not a birthmark")
    family = family_of(path, metadata["birthmark_type"])
    return birthmark, family, metadata.get("ir", "ghidra-pcode")


def document_frequencies(paths):
    """How many functions contain each element, how many functions there
    are, and the one family and IR every input shares."""
    df = Counter()
    n = 0
    kinds = {}
    for path in paths:
        birthmark, family, ir = read(path)
        kinds.setdefault((family, ir), path)
        if len(kinds) > 1:
            (a, pa), (b, pb) = list(kinds.items())[:2]
            raise Refused(f"{pa} is {a[0]} on {a[1]} and {pb} is {b[0]} on "
                          f"{b[1]}; convert one family and IR at a time")
        for function in birthmark["elements"]:
            df.update(term_frequencies(path, function["data"]).keys())
            n += 1
    (family, ir), = kinds
    return df, n, family, ir


def idf(df, n):
    return math.log((1 + n) / (1 + df)) + 1


def convert(path, idfs, family, scale):
    birthmark, _, _ = read(path)
    for function in birthmark["elements"]:
        tf = term_frequencies(path, function["data"])
        weights = {e: round(c * idfs[e] * scale) for e, c in tf.items()}
        # Sorted as oinkie writes a frequency birthmark, so that converting
        # the same inputs twice writes the same bytes.
        if family.endswith("gram"):
            function["data"] = {
                "KgramFreq": [[list(e), w] for e, w in sorted(weights.items())]
            }
        else:
            function["data"] = {"Freq": dict(sorted(weights.items()))}
    birthmark["metadata"]["birthmark_type"] = freq_type(family)
    return birthmark


def write_idf(path, df, n, idfs):
    with open(path, "w", encoding="utf-8", newline="") as f:
        out = csv.writer(f)
        out.writerow(["element", "df", "idf"])
        out.writerow(["(functions)", n, ""])
        for e in sorted(df):
            out.writerow([" ".join(e) if isinstance(e, tuple) else e, df[e], idfs[e]])


def main():
    parser = argparse.ArgumentParser(
        description="Reweight seq or freq birthmarks by TF-IDF over all of them, "
                    "writing frequency birthmarks.")
    parser.add_argument("input_dir", type=Path, help="directory of birthmark JSON files")
    parser.add_argument("output_dir", type=Path, help="directory to write the reweighted birthmarks to")
    parser.add_argument("--scale", type=int, default=1_000_000,
                        help="multiplier applied before rounding a weight to a whole count "
                             "(default: 1000000)")
    args = parser.parse_args()

    if args.scale < 1:
        parser.error("--scale must be at least 1")
    if args.input_dir.resolve() == args.output_dir.resolve():
        parser.error("the output directory must differ from the input directory, "
                     "whose files would be overwritten")
    paths = sorted(args.input_dir.glob("*.json"))
    if not paths:
        parser.error(f"{args.input_dir}: no birthmark JSON files")

    try:
        df, n, family, ir = document_frequencies(paths)
        idfs = {e: idf(d, n) for e, d in df.items()}
        args.output_dir.mkdir(parents=True, exist_ok=True)
        for path in paths:
            birthmark = convert(path, idfs, family, args.scale)
            with open(args.output_dir / path.name, "w", encoding="utf-8") as f:
                json.dump(birthmark, f, ensure_ascii=False, indent=2)
                f.write("\n")
        write_idf(args.output_dir / "idf.csv", df, n, idfs)
    except Refused as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    print(f"{len(paths)} birthmarks, {n} functions, {len(df)} elements "
          f"({family} on {ir}) -> {args.output_dir}", file=sys.stderr)


if __name__ == "__main__":
    main()
