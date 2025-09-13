#!/usr/bin/env python3

import json
import sys
from collections import defaultdict

def find_intrinsics_in_body(body, test_name):
    """Find all intrinsic calls in a function body"""
    intrinsics = []
    
    if not body:
        return intrinsics
    
    # Check blocks for intrinsic calls
    for block in body.get('blocks', []):
        # Check terminator
        terminator = block.get('terminator', {})
        if terminator.get('kind', {}).get('Call'):
            call = terminator['kind']['Call']
            func = call.get('func', '')
            
            # Check if it's a known intrinsic function
            if isinstance(func, str):
                if any(intrinsic in func for intrinsic in ['black_box', 'raw_eq', 'assert_inhabited', 'intrinsic', 'copy_nonoverlapping']):
                    intrinsics.append({
                        'type': 'terminator',
                        'func': func,
                        'span': terminator.get('span', 'unknown')
                    })
        
        # Check statements  
        for stmt in block.get('statements', []):
            if stmt.get('kind', {}).get('Call'):
                call = stmt['kind']['Call']
                func = call.get('func', '')
                
                if isinstance(func, str):
                    if any(intrinsic in func for intrinsic in ['black_box', 'raw_eq', 'assert_inhabited', 'intrinsic', 'copy_nonoverlapping']):
                        intrinsics.append({
                            'type': 'statement',
                            'func': func,
                            'span': stmt.get('span', 'unknown')
                        })
    
    return intrinsics

def analyze_smir(filename):
    with open(filename, 'r') as f:
        data = json.load(f)
    
    test_intrinsics = defaultdict(list)
    all_intrinsics = set()
    
    # Analyze all test functions
    for item in data.get('items', []):
        name = item.get('name', '')
        
        # Focus on test functions
        if 'test_' in name and 'entrypoint' in name:
            body = item.get('body')
            if body:
                intrinsics = find_intrinsics_in_body(body, name)
                if intrinsics:
                    test_intrinsics[name] = intrinsics
                    for intr in intrinsics:
                        # Extract intrinsic name from function path
                        func = intr['func']
                        if 'black_box' in func:
                            all_intrinsics.add('black_box')
                        elif 'raw_eq' in func:
                            all_intrinsics.add('raw_eq')
                        elif 'assert_inhabited' in func:
                            all_intrinsics.add('assert_inhabited')
                        elif 'copy_nonoverlapping' in func:
                            all_intrinsics.add('copy_nonoverlapping')
                        elif 'intrinsic' in func:
                            all_intrinsics.add(func.split('::')[-1])
    
    # Check all functions for intrinsic symbols
    functions = data.get('functions', [])
    if isinstance(functions, list):
        for func_info in functions:
            if isinstance(func_info, list) and len(func_info) > 1:
                if isinstance(func_info[1], dict):
                    sym = func_info[1].get('NormalSym', '')
                    if any(intrinsic in sym for intrinsic in ['black_box', 'raw_eq', 'assert_inhabited', 'intrinsic']):
                        # Extract intrinsic name
                        if 'black_box' in sym:
                            all_intrinsics.add('black_box')
                        elif 'raw_eq' in sym:
                            all_intrinsics.add('raw_eq')
                        elif 'assert_inhabited' in sym:
                            all_intrinsics.add('assert_inhabited')
                        elif 'copy_nonoverlapping' in sym:
                            all_intrinsics.add('copy_nonoverlapping')
    
    print("=" * 80)
    print("INTRINSICS ANALYSIS FOR P-TOKEN VERIFICATION")
    print("=" * 80)
    
    print("\n## Summary of All Intrinsics Found:")
    print("-" * 40)
    for intrinsic in sorted(all_intrinsics):
        print(f"  - {intrinsic}")
    
    print(f"\nTotal unique intrinsics: {len(all_intrinsics)}")
    
    print("\n## Tests Using Intrinsics:")
    print("-" * 40)
    
    if not test_intrinsics:
        print("No direct intrinsic calls found in test functions.")
    else:
        for test_name, intrinsics in sorted(test_intrinsics.items()):
            print(f"\n### {test_name}")
            for intr in intrinsics:
                print(f"  - Type: {intr['type']}")
                print(f"    Function: {intr['func']}")
                print(f"    Location: {intr['span']}")
    
    # Special check for test_process_get_account_data_size
    print("\n## Special Analysis: test_process_get_account_data_size")
    print("-" * 40)
    
    for item in data.get('items', []):
        if 'test_process_get_account_data_size' in item.get('name', ''):
            print(f"Found test: {item['name']}")
            body = item.get('body')
            if body:
                # Count blocks and statements
                num_blocks = len(body.get('blocks', []))
                num_statements = sum(len(block.get('statements', [])) for block in body.get('blocks', []))
                print(f"  - Number of blocks: {num_blocks}")
                print(f"  - Number of statements: {num_statements}")
                
                # Look for any function calls that might be intrinsics
                for block_idx, block in enumerate(body.get('blocks', [])):
                    terminator = block.get('terminator', {})
                    if terminator.get('kind', {}).get('Call'):
                        call = terminator['kind']['Call']
                        func = call.get('func', '')
                        if isinstance(func, str):
                            # Check for any interesting function
                            if 'owner' in func or 'eq' in func or 'cmp' in func:
                                print(f"  - Block {block_idx}: Calls {func}")

if __name__ == '__main__':
    analyze_smir('artefacts/p-token.smir.json')