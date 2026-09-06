import json
from collections import Counter
import sys

def analyze_json(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
            
        functions = data.get('functions', [])
        num_functions = len(functions)
        
        # 全ての関数の全 op をフラットなリストに抽出
        all_ops = []
        for func in functions:
            ops_in_func = [item['op'] for item in func.get('ops', [])]
            all_ops.extend(ops_in_func)
            
        # 頻度を集計
        op_counts = Counter(all_ops)
        
        # 結果の出力
        print(f"File: {file_path}")
        print(f"Number of functions: {num_functions}")
        print("-" * 30)
        print(f"{'Operation':<15} | {'Count':<10}")
        print("-" * 30)
        
        # 頻度の高い順にソートして表示
        for op, count in op_counts.most_common():
            print(f"{op:<15} | {count:<10}")
            
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        target_file = sys.argv[1]
        analyze_json(target_file)
    else:
        print("Usage: analyze_ops <pcode_json>")
