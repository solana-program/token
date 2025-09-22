#!/usr/bin/env python3
"""
Simple Proof Tree Visualization Script
"""

import re
import sys
from collections import defaultdict


def main():
    if len(sys.argv) != 2:
        print("Usage: python3 visualize_proof_tree.py <proof-file.txt>", file=sys.stderr)
        return 1

    proof_file = sys.argv[1]

    try:
        print("🌲 PROOF TREE STRUCTURE")
        print("═" * 80)
        print()
        print("Format: Node ID | Type | Attributes | Line Number")
        print("─" * 80)
        print()

        # Track statistics
        node_stats = defaultdict(int)

        with open(proof_file, 'r', encoding='utf-8') as f:
            for line_num, line in enumerate(f, 1):
                # Match lines with node numbers
                match = re.search(r'([\s┌├└│┃]*[├─└┌]─)\s*(\d+)\s*(.*)', line)
                if match:
                    prefix, node_id, attributes = match.groups()

                    # Type indicator
                    type_indicator = ""
                    if 'root' in attributes:
                        type_indicator = "🌳 ROOT"
                        node_stats['root'] += 1
                    elif 'split' in attributes:
                        type_indicator = "🔀 SPLIT"
                        node_stats['split'] += 1
                    elif 'stuck' in attributes:
                        type_indicator = "❌ STUCK"
                        node_stats['stuck'] += 1
                    elif 'pending' in attributes:
                        type_indicator = "⏳ PENDING"
                        node_stats['pending'] += 1
                    elif 'terminal' in attributes:
                        type_indicator = "✅ TERMINAL"
                        node_stats['terminal'] += 1
                    elif 'leaf' in attributes:
                        type_indicator = "🍃 LEAF"
                        node_stats['leaf'] += 1
                    else:
                        type_indicator = "   NORMAL"
                        node_stats['normal'] += 1

                    # Clean attributes for display
                    attr_display = f"({attributes})" if attributes else ""

                    # Format and display
                    print(f"{prefix} {node_id:2s} {type_indicator:11s} {attr_display:20s} │ line {line_num:5d}")

        print()
        print("📊 STATISTICS")
        print("═" * 40)

        total_nodes = sum(node_stats.values())
        print(f"📈 Total nodes: {total_nodes}")
        print()

        # Node type breakdown with icons
        type_icons = {
            "root": "🌳",
            "split": "🔀",
            "stuck": "❌",
            "pending": "⏳",
            "terminal": "✅",
            "leaf": "🍃",
            "normal": "⚪"
        }

        for node_type, count in sorted(node_stats.items()):
            icon = type_icons.get(node_type, "❓")
            print(f"  {icon} {node_type.title():12s}: {count:3d}")

        # Proof outcome
        print()
        print("🎯 Proof Outcome:")
        if node_stats['terminal'] > 0:
            print(f"  ✅ SUCCESS: Found {node_stats['terminal']} terminal state(s)")
        if node_stats['stuck'] > 0:
            print(f"  ❌ STUCK: {node_stats['stuck']} branch(es) got stuck")
        if node_stats['pending'] > 0:
            print(f"  ⏳ PENDING: {node_stats['pending']} branch(es) still pending")

        return 0

    except FileNotFoundError:
        print(f"File not found: {proof_file}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())