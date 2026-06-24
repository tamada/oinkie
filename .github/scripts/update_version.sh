#! /bin/sh

FROM_VERSION=`grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/g'`
TO_VERSION=$1

sed "s/^version = \".*\"/version = \"$TO_VERSION\"/" Cargo.toml > a ; mv a Cargo.toml
sed "s/\${FROM_VERSION}/$TO_VERSION/g" README.md > a ; mv a README.md
sed "s/\${FROM_VERSION}/$TO_VERSION/g" docs/content/_index.md > a ; mv a docs/content/_index.md
