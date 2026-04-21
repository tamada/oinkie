if [ -z "$1" ]; then
    echo "Usage: $0 <base>"
    exit 1
fi
base=$(cd $1; pwd)

mkdir -p $base/{images,results,tables}
for i in cosine dice levenshtein lcs jaccard euclidean simpson weighted-jaccard
do
    echo "Processing $i..."
    cp $base/$i/results.csv $base/results/$i.csv
    python scripts/results2table.py $base/results/$i.csv > $base/tables/$i.csv
    heatman -p 10 --order $base/../headers.csv --dest $base/images/$i.png $base/tables/$i.csv
done