#!/usr/bin/env python3

import json
import sys
import re
from collections import defaultdict, Counter
from pathlib import Path

def extract_intrinsic_name(func_symbol):
    """Extract the intrinsic name from a mangled function symbol"""
    intrinsics = {
        'black_box': 'black_box',
        'raw_eq': 'raw_eq',
        'assert_inhabited': 'assert_inhabited',
        'assert_zero_valid': 'assert_zero_valid',
        'assert_mem_uninitialized_valid': 'assert_mem_uninitialized_valid',
        'copy_nonoverlapping': 'copy_nonoverlapping',
        'write_bytes': 'write_bytes',
        'cold_path': 'cold_path',
        'unlikely': 'unlikely',
        'likely': 'likely',
        'assume': 'assume',
        'unreachable': 'unreachable',
        'abort': 'abort',
        'forget': 'forget',
        'transmute': 'transmute',
        'size_of': 'size_of',
        'align_of': 'align_of',
        'type_id': 'type_id',
        'type_name': 'type_name',
        'bswap': 'bswap',
        'bitreverse': 'bitreverse',
        'ctpop': 'ctpop',
        'ctlz': 'ctlz',
        'cttz': 'cttz',
        'rotate_left': 'rotate_left',
        'rotate_right': 'rotate_right',
        'saturating_add': 'saturating_add',
        'saturating_sub': 'saturating_sub',
        'wrapping_add': 'wrapping_add',
        'wrapping_sub': 'wrapping_sub',
        'wrapping_mul': 'wrapping_mul',
        'exact_div': 'exact_div',
        'unchecked_add': 'unchecked_add',
        'unchecked_sub': 'unchecked_sub',
        'unchecked_mul': 'unchecked_mul',
        'unchecked_div': 'unchecked_div',
        'unchecked_rem': 'unchecked_rem',
        'unchecked_shl': 'unchecked_shl',
        'unchecked_shr': 'unchecked_shr',
        'float_to_int_unchecked': 'float_to_int_unchecked',
        'discriminant_value': 'discriminant_value',
        'ptr_guaranteed_cmp': 'ptr_guaranteed_cmp',
        'ptr_offset_from': 'ptr_offset_from',
        'ptr_offset_from_unsigned': 'ptr_offset_from_unsigned',
        'is_val_statically_known': 'is_val_statically_known',
        'const_eval_select': 'const_eval_select',
        'vtable_size': 'vtable_size',
        'vtable_align': 'vtable_align',
        'min_align_of_val': 'min_align_of_val',
        'size_of_val': 'size_of_val'
    }
    
    for key in intrinsics:
        if key in func_symbol.lower():
            return intrinsics[key]
    return None

def analyze_proof_output(proof_file):
    """Analyze proof output file for stuck intrinsics"""
    stuck_intrinsics = set()
    
    if not proof_file.exists():
        return stuck_intrinsics
        
    content = proof_file.read_text()
    
    # Look for #execIntrinsic patterns
    intrinsic_pattern = r'#execIntrinsic\s*\(\s*symbol\s*\(\s*"([^"]+)"\s*\)'
    matches = re.findall(intrinsic_pattern, content)
    stuck_intrinsics.update(matches)
    
    # Look for stuck nodes with intrinsic mentions
    if 'raw_eq' in content and 'pending' in content:
        stuck_intrinsics.add('raw_eq')
    if 'assert_inhabited' in content and 'pending' in content:
        stuck_intrinsics.add('assert_inhabited')
        
    return stuck_intrinsics

def analyze_smir(filename):
    """Comprehensive intrinsics analysis for entire program verification"""
    with open(filename, 'r') as f:
        data = json.load(f)
    
    # Track all intrinsics
    all_intrinsics = set()
    test_intrinsics = defaultdict(set)
    function_intrinsics = defaultdict(set)
    intrinsic_usage_count = Counter()
    
    # Analyze functions list for intrinsic symbols
    functions = data.get('functions', [])
    if isinstance(functions, list):
        for func_info in functions:
            if isinstance(func_info, list) and len(func_info) > 1:
                if isinstance(func_info[1], dict):
                    sym = func_info[1].get('NormalSym', '')
                    intrinsic = extract_intrinsic_name(sym)
                    if intrinsic:
                        all_intrinsics.add(intrinsic)
                        intrinsic_usage_count[intrinsic] += 1
    
    # Analyze all items (functions) in detail
    for item in data.get('items', []):
        name = item.get('name', '')
        body = item.get('body')
        
        if not body:
            continue
            
        # Track intrinsics per function
        func_intrinsics = set()
        
        for block in body.get('blocks', []):
            # Check terminator
            terminator = block.get('terminator', {})
            term_kind = terminator.get('kind', {})
            
            # Check for Call terminators
            if term_kind.get('Call'):
                call = term_kind['Call']
                func = call.get('func', '')
                if isinstance(func, str):
                    intrinsic = extract_intrinsic_name(func)
                    if intrinsic:
                        func_intrinsics.add(intrinsic)
                        intrinsic_usage_count[intrinsic] += 1
            
            # Check statements
            for stmt in block.get('statements', []):
                stmt_kind = stmt.get('kind', {})
                if stmt_kind.get('Call'):
                    call = stmt_kind['Call']
                    func = call.get('func', '')
                    if isinstance(func, str):
                        intrinsic = extract_intrinsic_name(func)
                        if intrinsic:
                            func_intrinsics.add(intrinsic)
                            intrinsic_usage_count[intrinsic] += 1
        
        if func_intrinsics:
            function_intrinsics[name] = func_intrinsics
            all_intrinsics.update(func_intrinsics)
            
            # Special tracking for test functions
            if 'test_' in name and 'entrypoint' in name:
                test_intrinsics[name] = func_intrinsics
    
    # Analyze proof outputs for stuck intrinsics
    proof_dir = Path('artefacts/proof')
    stuck_intrinsics_by_test = {}
    
    if proof_dir.exists():
        for proof_file in proof_dir.glob('*.json'):
            test_name = proof_file.stem.replace('p-token.smir.', '')
            stuck = analyze_proof_output(proof_file)
            if stuck:
                stuck_intrinsics_by_test[test_name] = stuck
    
    return {
        'all_intrinsics': all_intrinsics,
        'test_intrinsics': test_intrinsics,
        'function_intrinsics': function_intrinsics,
        'intrinsic_usage_count': intrinsic_usage_count,
        'stuck_intrinsics_by_test': stuck_intrinsics_by_test
    }

def check_mir_semantics_support():
    """Check which intrinsics are implemented in mir-semantics"""
    implemented = set()
    mir_semantics_path = Path('mir-semantics/kmir/src/kmir/kdist/mir-semantics')
    
    if mir_semantics_path.exists():
        # Search for intrinsic implementations
        for k_file in mir_semantics_path.rglob('*.md'):
            content = k_file.read_text()
            if 'intrinsic' in content.lower():
                # Look for rule definitions
                if 'rule [intrinsic-' in content or 'rule #execIntrinsic' in content:
                    # Extract implemented intrinsics
                    if 'black_box' in content:
                        implemented.add('black_box')
                    if 'copy_nonoverlapping' in content:
                        implemented.add('copy_nonoverlapping')
                    # Add more as we find them
    
    return implemented

def generate_report(analysis):
    """Generate comprehensive intrinsics report"""
    all_intrinsics = analysis['all_intrinsics']
    test_intrinsics = analysis['test_intrinsics']
    usage_count = analysis['intrinsic_usage_count']
    stuck_by_test = analysis['stuck_intrinsics_by_test']
    
    # Check what's implemented
    implemented = check_mir_semantics_support()
    
    # Categorize intrinsics
    missing = all_intrinsics - implemented
    blocking = set()
    for stuck_set in stuck_by_test.values():
        blocking.update(stuck_set)
    
    print("=" * 100)
    print("COMPREHENSIVE INTRINSICS ANALYSIS FOR P-TOKEN (entrypoint.rs) VERIFICATION")
    print("=" * 100)
    
    print("\n## SUMMARY")
    print("-" * 50)
    print(f"Total unique intrinsics found: {len(all_intrinsics)}")
    print(f"Implemented in K semantics: {len(implemented)}")
    print(f"Missing implementations: {len(missing)}")
    print(f"Currently blocking tests: {len(blocking)}")
    
    print("\n## ALL INTRINSICS REQUIRED")
    print("-" * 50)
    for intrinsic in sorted(all_intrinsics):
        status = "✅" if intrinsic in implemented else "❌"
        blocking_mark = " 🚫 BLOCKING" if intrinsic in blocking else ""
        count = usage_count[intrinsic]
        print(f"{status} {intrinsic:<30} (used {count} times){blocking_mark}")
    
    print("\n## MISSING INTRINSICS (Need Implementation)")
    print("-" * 50)
    if missing:
        priority_missing = []
        for intrinsic in sorted(missing):
            if intrinsic in blocking:
                priority_missing.append((intrinsic, "HIGH - Currently blocking tests"))
            else:
                priority_missing.append((intrinsic, "MEDIUM - May be needed"))
        
        for intrinsic, priority in priority_missing:
            print(f"- {intrinsic:<30} Priority: {priority}")
    else:
        print("None - all intrinsics are implemented")
    
    print("\n## INTRINSICS BY TEST FUNCTION")
    print("-" * 50)
    for test_name in sorted(test_intrinsics.keys()):
        if 'entrypoint::test_' in test_name:
            short_name = test_name.replace('entrypoint::', '')
            intrinsics = test_intrinsics[test_name]
            stuck = stuck_by_test.get(test_name, set())
            
            print(f"\n### {short_name}")
            for intrinsic in sorted(intrinsics):
                status = "✅" if intrinsic in implemented else "❌"
                stuck_mark = " (STUCK HERE)" if intrinsic in stuck else ""
                print(f"    {status} {intrinsic}{stuck_mark}")
    
    print("\n## BLOCKING ANALYSIS")
    print("-" * 50)
    if stuck_by_test:
        print("Tests stuck on missing intrinsics:")
        for test_name, stuck_intrinsics in stuck_by_test.items():
            if stuck_intrinsics:
                print(f"\n{test_name}:")
                for intrinsic in stuck_intrinsics:
                    print(f"  - {intrinsic}")
    else:
        print("No tests found stuck on intrinsics")
    
    print("\n## IMPLEMENTATION PRIORITY")
    print("-" * 50)
    print("Based on test blockage and usage frequency:\n")
    
    # Calculate priority
    priority_scores = {}
    for intrinsic in missing:
        score = usage_count[intrinsic] * 10  # Base score from usage
        if intrinsic in blocking:
            score += 1000  # High priority for blocking
        priority_scores[intrinsic] = score
    
    # Sort by priority
    sorted_priority = sorted(priority_scores.items(), key=lambda x: x[1], reverse=True)
    
    for i, (intrinsic, score) in enumerate(sorted_priority[:10], 1):
        blocking_status = "BLOCKING" if intrinsic in blocking else "Not blocking"
        print(f"{i}. {intrinsic:<25} (Score: {score}, Usage: {usage_count[intrinsic]}, Status: {blocking_status})")
    
    print("\n## INTRINSIC CATEGORIES")
    print("-" * 50)
    
    categories = {
        'Memory Operations': ['copy_nonoverlapping', 'write_bytes', 'transmute'],
        'Pointer Operations': ['raw_eq', 'ptr_guaranteed_cmp', 'ptr_offset_from', 'ptr_offset_from_unsigned'],
        'Type Operations': ['assert_inhabited', 'assert_zero_valid', 'assert_mem_uninitialized_valid', 
                          'size_of', 'align_of', 'type_id', 'type_name', 'discriminant_value'],
        'Arithmetic': ['saturating_add', 'saturating_sub', 'wrapping_add', 'wrapping_sub', 
                      'wrapping_mul', 'unchecked_add', 'unchecked_sub', 'unchecked_mul',
                      'unchecked_div', 'unchecked_rem', 'exact_div'],
        'Bit Operations': ['bswap', 'bitreverse', 'ctpop', 'ctlz', 'cttz', 
                          'rotate_left', 'rotate_right', 'unchecked_shl', 'unchecked_shr'],
        'Control Flow': ['cold_path', 'unlikely', 'likely', 'assume', 'unreachable', 'abort'],
        'Debug/Test': ['black_box', 'forget'],
        'Other': ['float_to_int_unchecked', 'const_eval_select', 'is_val_statically_known',
                 'vtable_size', 'vtable_align', 'min_align_of_val', 'size_of_val']
    }
    
    for category, intrinsics in categories.items():
        found = [i for i in intrinsics if i in all_intrinsics]
        if found:
            print(f"\n{category}:")
            for intrinsic in found:
                status = "✅" if intrinsic in implemented else "❌"
                print(f"  {status} {intrinsic}")
    
    return {
        'total': len(all_intrinsics),
        'implemented': len(implemented),
        'missing': len(missing),
        'blocking': len(blocking),
        'missing_list': sorted(missing),
        'blocking_list': sorted(blocking)
    }

if __name__ == '__main__':
    print("Analyzing p-token.smir.json for all intrinsics...")
    analysis = analyze_smir('artefacts/p-token.smir.json')
    summary = generate_report(analysis)
    
    print("\n" + "=" * 100)
    print("FINAL SUMMARY FOR VERIFICATION")
    print("=" * 100)
    print(f"""
To fully verify entrypoint.rs, you need to implement {summary['missing']} intrinsics.
Priority intrinsics that are currently blocking tests: {', '.join(summary['blocking_list']) if summary['blocking_list'] else 'None'}

The most critical missing intrinsics are those that appear in multiple tests and are
currently causing proof failures. Implementing these will unblock formal verification.
""")