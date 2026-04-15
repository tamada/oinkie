if [ -z "$1" ]; then
    echo "Usage: $0 <base>"
    exit 1
fi
$base=$1
mkdir -p $1/{images,results,tables}
for i in cosine dice levenshtein lcs jaccard euclidean simpson weighted-jaccard
do
    echo "Processing $i..."
    cp $base/$i/results.json $base/results/$i.json
    python scripts/results2table.py $base/results/$i.json > $base/tables/$i.csv
    heatman -p 10 --order $i/../headers.csv --dest $base/images/$i.png $base/tables/$i.csv
done