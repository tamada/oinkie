import json
import sys
from collections import Counter

def analyze_function_sizes(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
            
        functions = data.get('functions', [])
        num_functions = len(functions)
        
        # 各関数に含まれる ops の数をリスト化
        sizes = [len(func.get('ops', [])) for func in functions]
        
        # サイズの頻度（同じ命令数を持つ関数がいくつあるか）を集計
        size_counts = Counter(sizes)
        
        print(f"File: {file_path}")
        print(f"Total number of functions: {num_functions}")
        print("-" * 40)
        print(f"{'Ops Count':<15} | {'Number of Functions':<20}")
        print("-" * 40)
        
        # 命令数が少ない順に表示
        for size in sorted(size_counts.keys()):
            print(f"{size:<15} | {size_counts[size]:<20}")
            
        # 統計情報の補足
        if sizes:
            print("-" * 40)
            print(f"Max size: {max(sizes)} ops")
            print(f"Min size: {min(sizes)} ops")
            print(f"Average size: {sum(sizes)/len(sizes):.2f} ops")

    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        target_file = sys.argv[1]
        analyze_function_sizes(target_file)
    else:
        print("Usage: python analyze_function_sizes.py <pcode_json>")
