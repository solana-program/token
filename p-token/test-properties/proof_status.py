#!/usr/bin/env python3
"""
Script to analyze proof txt files and output statistics
"""

import os
import re
import glob
from pathlib import Path

def extract_steps_from_txt(txt_file):
    """Extract steps count from a proof txt file"""
    try:
        with open(txt_file, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Find all "(X steps)" patterns and sum them up
        steps_matches = re.findall(r'\((\d+)\s+steps\)', content)
        if steps_matches:
            total_steps = sum(int(step) for step in steps_matches)
            return total_steps
        
        return None
    except Exception as e:
        print(f"Error reading {txt_file}: {e}")
        return None

def extract_status_from_txt(txt_file):
    """Extract status from a proof txt file"""
    try:
        with open(txt_file, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Look for status indicators
        # Check if the proof reaches a terminal state (PASSED)
        if '(leaf, target, terminal)' in content:
            return 'PASSED'
        elif '(stuck' in content:
            return 'Stuck'
        elif '(pending' in content:
            return 'Pending'
        elif 'timeout' in content.lower():
            return 'timeout'
        
        return None
    except Exception as e:
        print(f"Error reading status from {txt_file}: {e}")
        return None

def extract_last_state_content(txt_file):
    """Extract the state content from the last steps node"""
    try:
        with open(txt_file, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Find all "(X steps)" patterns
        steps_matches = re.finditer(r'\((\d+)\s+steps\)', content)
        steps_positions = [(m.start(), m.group(1)) for m in steps_matches]
        
        if not steps_positions:
            return None
        
        # Get the position of the last steps
        last_steps_pos, last_steps_count = steps_positions[-1]
        
        # Find the node that contains the last steps
        # Look for the node pattern that comes right after the last steps
        content_after_last_steps = content[last_steps_pos:]
        
        # Find all node patterns after the last steps
        # Look for both └─ and ├─ patterns, with or without status
        node_matches = re.finditer(r'[└├]─ \d+(?: \((stuck, leaf|leaf, target, terminal|leaf, pending)\))?', content_after_last_steps)
        node_positions = [(m.start(), m.group(0)) for m in node_matches]
        
        if not node_positions:
            return None
        
        # Extract <k> content from all nodes
        all_k_contents = []
        for node_start, node_text in node_positions:
            # Find the <k> content within this node
            node_content = content_after_last_steps[node_start:]
            
            # Find the <k> tag within this node
            k_start = node_content.find('<k>')
            if k_start == -1:
                continue
            
            # Find the corresponding </k> tag
            k_end = node_content.find('</k>', k_start)
            if k_end == -1:
                continue
            
            # Extract the content between <k> and </k>
            k_content = node_content[k_start:k_end + 4]  # Include the tags
            
            # Clean up the content - remove excessive whitespace and newlines
            k_content = re.sub(r'\n\s*\n', '\n', k_content)  # Remove empty lines
            k_content = re.sub(r'^\s+', '', k_content, flags=re.MULTILINE)  # Remove leading whitespace
            
            all_k_contents.append(f"Node {node_text}: {k_content.strip()}")
        
        if all_k_contents:
            return "\n\n".join(all_k_contents)
        
        return None
        
    except Exception as e:
        print(f"Error extracting state content from {txt_file}: {e}")
        return None

def main():
    # Define proof directory
    script_dir = Path(__file__).parent
    proof_dir = script_dir / "artefacts" / "proof"
    
    print(f"Analyzing proof files in: {proof_dir}")
    
    # Check if directory exists
    if not proof_dir.exists():
        print(f"Error: {proof_dir} not found!")
        return
    
    # Get all -full.txt files
    txt_files = glob.glob(os.path.join(proof_dir, "*-full.txt"))
    
    if not txt_files:
        print("No -full.txt files found!")
        return
    
    print(f"Found {len(txt_files)} proof files")
    print()
    
    # Analyze each file
    results = []
    for txt_file in sorted(txt_files):
        basename = os.path.basename(txt_file)
        test_name = basename[:-9]  # Remove '-full.txt'
        
        steps = extract_steps_from_txt(txt_file)
        status = extract_status_from_txt(txt_file)
        state_content = extract_last_state_content(txt_file)
        
        results.append({
            'name': test_name,
            'steps': steps,
            'status': status,
            'state_content': state_content
        })
    
    # Print results in markdown format
    for result in results:
        name = result['name']
        steps = str(result['steps']) if result['steps'] is not None else 'N/A'
        status = result['status'] if result['status'] is not None else 'Unknown'
        state_content = result['state_content'] if result['state_content'] else 'No state content found'
        
        print(f"## {name} - {status} - {steps}")
        print(state_content)
        print()

if __name__ == "__main__":
    main()
