#! /usr/bin/env python

import csv
import sys
import os
from collections import defaultdict

def format_duration(nsec):
    """nsec を 分、秒、コンマ区切りnsec の形式に変換する"""
    seconds = nsec / 1_000_000_000
    minutes = int(seconds // 60)
    remaining_seconds = seconds % 60
    # 0m 0.00s (000,000,000 nsec) の形式
    return f"{minutes}m {remaining_seconds:.2f}s ({nsec:,} nsec)"

def create_similarity_matrix(input_file):
    if not os.path.exists(input_file):
        print(f"Error: {input_file} not found.", file=sys.stderr)
        return

    # matrix_data[(a, b)] = similarity の形で保存
    matrix_data = {}
    sources = set()
    total_duration = 0

    # 1. CSVの読み込み
    with open(input_file, mode='r', encoding='utf-8') as f:
        reader = csv.reader(f)
        for row in reader:
            if not row or len(row) < 5: continue
            
            try:
                similarity = float(row[1])
                # パスからファイル名を抽出
                src_a = row[2].split('/')[-1]
                src_b = row[3].split('/')[-1]
                duration = int(row[4])
                
                total_duration += duration
                # どちらの順序で来ても参照できるようタプルをソートしてキーにする
                pair = tuple(sorted([src_a, src_b]))
                matrix_data[pair] = similarity
                
                sources.add(src_a)
                sources.add(src_b)
            except (ValueError, IndexError):
                continue

    # ソース名をソート
    sorted_sources = sorted(list(sources))

    # 2. 標準出力へCSV形式で書き出し
    writer = csv.writer(sys.stdout)
    
    # ヘッダー行
    writer.writerow([""] + sorted_sources)
    
    # データ行 (完全な右上三角行列)
    for i, src_a in enumerate(sorted_sources):
        row = [src_a]
        for j, src_b in enumerate(sorted_sources):
            # i <= j (対角成分を含む右上) の場合のみ表示
            if i <= j:
                pair = tuple(sorted([src_a, src_b]))
                val = matrix_data.get(pair, "")
                if val != "":
                    row.append(f"{val:.2f}")
                else:
                    row.append("")
            else:
                # 左下部分は完全に空にする
                row.append("")
        writer.writerow(row)
    
    # 3. 合計時間の出力
    writer.writerow([])
    writer.writerow(["total duration", format_duration(total_duration)])

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: python {sys.argv[0]} <input_csv_file>", file=sys.stderr)
        sys.exit(1)
    
    create_similarity_matrix(sys.argv[1])
