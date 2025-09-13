# SMIR Intrinsics Discovery Report

**Source File:** `artefacts/p-token.smir.json`  
**Analysis Date:** /Users/steven/Desktop/projs/solana-token/p-token/test-properties  
**Total Intrinsics Discovered:** 35  
**Total Occurrences:** 328  
**Discovery Method:** Data-driven analysis (no hardcoded intrinsic lists)

---

## Executive Summary

This report presents a comprehensive data-driven analysis of intrinsic function usage
in the SMIR JSON file. Unlike traditional approaches that rely on predefined lists,
this analysis discovers intrinsics purely from the JSON structure and content patterns.

### Key Findings

**Most Frequently Referenced Intrinsics:**

- `Intrinsic`: 154 occurrences
- `intrinsic`: 110 occurrences
- `IntrinsicSym`: 60 occurrences
- `core::hint`: 9 occurrences
- `core::intrinsics`: 3 occurrences

**Discovery Statistics:**
- Unique intrinsics found: 35
- Namespace categories: 5
- Pattern types discovered: 2
- JSON structure patterns: 0

---

## Discovery Methodology

This analysis employed a multi-phase discovery approach:

1. **Structural Pattern Discovery**: Analyzed JSON structure for intrinsic-related keys and patterns
2. **Deep JSON Traversal**: Recursively examined all JSON elements for intrinsic references
3. **Content Pattern Matching**: Applied regex patterns to identify intrinsic usage patterns
4. **Symbol Analysis**: Extracted and decoded mangled symbol names
5. **Categorization**: Grouped discoveries by namespace and inferred functionality
6. **Frequency Analysis**: Counted occurrences to identify most important intrinsics

All intrinsic names were discovered from the data itself - no predefined lists were used.

---

## All Discovered Intrinsics

| Intrinsic Name | Frequency | Primary Context |
|----------------|-----------|-----------------|
| `Intrinsic` | 154 | dict_entry |
| `intrinsic` | 110 | dict_entry |
| `IntrinsicSym` | 60 | dict_entry |
| `core::hint` | 9 | dict_entry |
| `core::intrinsics` | 3 | dict_entry |
| `panicking` | 1 | unknown |
| `num` | 1 | unknown |
| `slic` | 1 | unknown |
| `cmp` | 1 | unknown |
| `str` | 1 | unknown |
| `fmt` | 1 | unknown |
| `ptr` | 1 | unknown |
| `_ZN11five8_const12decode_const` | 1 | dict_entry |
| `_ZN4core3fmt9Arguments9new_const` | 1 | dict_entry |
| `_ZN4core10intrinsics19copy_nonoverlapping18precondition_check` | 0 | unknown |
| `black_box` | 0 | unknown |
| `core::intrinsics::copy_nonoverlapping::precondition_check` | 0 | unknown |
| `cold_path` | 0 | unknown |
| `hint` | 0 | unknown |
| `cor` | 0 | unknown |
| `char` | 0 | unknown |
| `core::intrinsics::cold_path` | 0 | unknown |
| `ub_ch` | 0 | unknown |
| `_ZN4core10intrinsics9cold_path` | 0 | unknown |
| `ops` | 0 | unknown |
| `_ZN4core10intrinsics19copy_nonoverlapping` | 0 | unknown |
| `core::intrinsics::copy_nonoverlapping::<core::mem::MaybeUninit<u8>>` | 0 | unknown |
| `copy_nonoverlapping` | 0 | unknown |
| `intrinsics` | 0 | unknown |
| `array` | 0 | unknown |
| `conv` | 0 | unknown |
| `unreachable_unchecked` | 0 | unknown |
| `copy_nonoverlappingprecondition_check` | 0 | unknown |
| `unreachable_uncheckedprecondition_check` | 0 | unknown |
| `option` | 0 | unknown |

---

## Namespace Analysis

Intrinsics categorized by their discovered namespace or inferred functionality:

### compiler_hints

**Count:** 1 intrinsics

- `black_box` (0 occurrences)

### core::intrinsics (mangled)

**Count:** 3 intrinsics

- `_ZN4core10intrinsics19copy_nonoverlapping18precondition_check` (0 occurrences)
- `_ZN4core10intrinsics9cold_path` (0 occurrences)
- `_ZN4core10intrinsics19copy_nonoverlapping` (0 occurrences)

### memory_operations

**Count:** 4 intrinsics

- `core::intrinsics::copy_nonoverlapping::precondition_check` (0 occurrences)
- `core::intrinsics::copy_nonoverlapping::<core::mem::MaybeUninit<u8>>` (0 occurrences)
- `copy_nonoverlapping` (0 occurrences)
- `copy_nonoverlappingprecondition_check` (0 occurrences)

### pointer_operations

**Count:** 1 intrinsics

- `ptr` (1 occurrences)

### uncategorized

**Count:** 26 intrinsics

- `Intrinsic` (154 occurrences)
- `intrinsic` (110 occurrences)
- `IntrinsicSym` (60 occurrences)
- `core::hint` (9 occurrences)
- `core::intrinsics` (3 occurrences)
- `panicking` (1 occurrences)
- `num` (1 occurrences)
- `slic` (1 occurrences)
- `cmp` (1 occurrences)
- `str` (1 occurrences)
- `fmt` (1 occurrences)
- `_ZN11five8_const12decode_const` (1 occurrences)
- `_ZN4core3fmt9Arguments9new_const` (1 occurrences)
- `cold_path` (0 occurrences)
- `hint` (0 occurrences)
- `cor` (0 occurrences)
- `char` (0 occurrences)
- `core::intrinsics::cold_path` (0 occurrences)
- `ub_ch` (0 occurrences)
- `ops` (0 occurrences)
- ... and 6 more

---

## Pattern Analysis

Patterns discovered during content analysis:

| Pattern Type | Description | Frequency | Examples |
|--------------|-------------|-----------|----------|
| quoted_intrinsic_strings | Pattern: "([^"]*intrinsic[^"]*)" | 132 | `IntrinsicSym`, `IntrinsicSym`, `IntrinsicSym` (+7 more) |
| core_mangled_symbols | Pattern: _ZN\d+core\d+([a-zA-Z_][^0-9E]+) | 612 | `fmt`, `panicking`, `num` (+7 more) |

---

## Frequency Analysis

Most frequently occurring intrinsics:

| Rank | Intrinsic | Occurrences | Percentage |
|------|-----------|-------------|------------|
| 1 | `Intrinsic` | 154 | 44.3% |
| 2 | `intrinsic` | 110 | 31.6% |
| 3 | `IntrinsicSym` | 60 | 17.2% |
| 4 | `core::hint` | 9 | 2.6% |
| 5 | `core::intrinsics` | 3 | 0.9% |
| 6 | `it` | 3 | 0.9% |
| 7 | `_ZN4core3fmt9Arguments9new_const` | 1 | 0.3% |
| 8 | `_ZN11five8_const12decode_const` | 1 | 0.3% |
| 9 | `fmt` | 1 | 0.3% |
| 10 | `panicking` | 1 | 0.3% |
| 11 | `num` | 1 | 0.3% |
| 12 | `slic` | 1 | 0.3% |
| 13 | `ptr` | 1 | 0.3% |
| 14 | `str` | 1 | 0.3% |
| 15 | `cmp` | 1 | 0.3% |

---

## Detailed Occurrence Analysis

Detailed information about where each intrinsic was found:

### `Intrinsic`

**Total occurrences:** 154

**dict_entry** (154 occurrences):
- `functions`
- `functions`
- `functions`
- ... and 151 more

### `intrinsic`

**Total occurrences:** 110

**dict_entry** (110 occurrences):
- `functions`
- `functions`
- `functions`
- ... and 107 more

### `IntrinsicSym`

**Total occurrences:** 50

**dict_entry** (50 occurrences):
- `functions`
- `functions`
- `functions`
- ... and 47 more

### `core::hint`

**Total occurrences:** 9

**dict_entry** (9 occurrences):
- `items`
- `items`
- `items`
- ... and 6 more

### `core::intrinsics`

**Total occurrences:** 3

**dict_entry** (3 occurrences):
- `items`
- `items`
- `items`

### `_ZN4core3fmt9Arguments9new_const`

**Total occurrences:** 1

**dict_entry** (1 occurrences):
- `functions`

### `_ZN11five8_const12decode_const`

**Total occurrences:** 1

**dict_entry** (1 occurrences):
- `items`

---

## Analysis Methodology

### Discovery Approach

This analysis used a completely data-driven approach with the following principles:

1. **No Hardcoded Lists**: No predefined intrinsic names or categories were used
2. **Pattern-Based Discovery**: Used regex patterns to identify intrinsic-related content
3. **Structural Analysis**: Analyzed JSON structure to find intrinsic usage patterns
4. **Symbol Decoding**: Decoded mangled symbols to extract intrinsic names
5. **Dynamic Categorization**: Categorized intrinsics based on discovered patterns

### Pattern Matching Strategy

The tool searched for various indicators of intrinsic usage:
- Keys containing "intrinsic", "IntrinsicSym", "Intrinsic", etc.
- Mangled symbol patterns like `_ZN.*core.*intrinsics.*`
- Namespace patterns like `core::intrinsics::*`
- Function declaration patterns

### Quality Assurance

- Names were cleaned to remove mangling artifacts
- Minimum length requirements applied to filter noise
- Frequency analysis to identify most significant intrinsics
- Context tracking to understand usage patterns

---

## Conclusions and Recommendations

### Summary

This data-driven analysis discovered **35 unique intrinsics** 
from the SMIR JSON file without using any predefined intrinsic lists. The analysis 
reveals the actual intrinsic usage patterns in the compiled code.

### Key Insights

1. **Comprehensive Discovery**: Found intrinsics across multiple namespaces and contexts
2. **Pattern-Based Approach**: Successfully identified intrinsics through structural patterns
3. **Frequency Analysis**: Identified the most critical intrinsics for implementation
4. **Context Awareness**: Tracked where and how intrinsics are used in the JSON structure

### Recommendations

1. **Implementation Priority**: Focus on the most frequently occurring intrinsics first
2. **Namespace Support**: Ensure support for all discovered namespaces
3. **Pattern Recognition**: Use discovered patterns to improve intrinsic detection in the future
4. **Testing Coverage**: Create tests covering all discovered intrinsic usage patterns

### Future Work

- Enhance pattern recognition for edge cases
- Add semantic analysis to better categorize intrinsics
- Develop automated testing based on discovered patterns
- Create mappings between SMIR intrinsics and their implementations

---

*This report was generated by the Data-Driven SMIR Intrinsics Discovery Tool*
*Analysis method: Pure data discovery with no hardcoded intrinsic knowledge*
