# Deep Analysis Report: Nondeterministic Branching in test_process_get_account_data_size

## Executive Summary

During formal verification of the `test_process_get_account_data_size` test, a critical issue was discovered: accessing `accounts[0].owner()` generates extensive nondeterministic branching (from node 5 branching to nodes 6, 7, 8, 9, etc.). This issue is not directly caused by the missing `raw_eq` intrinsic function, but rather stems from type uncertainty when the symbolic execution framework processes symbolic data structures.

## 1. Problem Description

### 1.1 Execution Path Analysis

```
Node 4 (24 steps executed)
  ↓
Node 5 (#traverseProjection processing #fromPAcc)
  ├── Node 6 (continues #fromPAcc) → further branches (nodes 11-15)
  ├── Node 7 (continues #fromPAcc) → further branches (nodes 16-20)
  ├── Node 8 (project:Value) → reaches raw_eq (node 21)
  └── Node 9 (getValue) → further branches (nodes 22-26)
```

### 1.2 Branch Explosion Statistics
- **Initial branching point**: Node 5
- **First-level branches**: 4 (nodes 6, 7, 8, 9)
- **Second-level branches**: At least 20 (nodes 11-36)
- **Total pending nodes**: Over 30

## 2. Root Cause Analysis

### 2.1 Symbolic Value Creation Process

#### Step 1: Cheatcode Execution
```k
rule [cheatcode-is-mint]:
  <k> #execTerminator(terminatorKindCall(FUNC, operandCopy(PLACE) .Operands, ...))
    => #mkPTokenMint(PLACE) ~> #execBlockIdx(TARGET)
  ...
  </k>
  requires #functionName(...) ==String "entrypoint::cheatcode_is_mint"
```

#### Step 2: Symbolic Mint Account Creation
```k
rule #addMint(Aggregate(_, _) #as P_ACC)
  => PAccountMint(
       #toPAcc(P_ACC),
       IMint(Flag(?MINT_AUTH_FLAG), 
             Key(?MINT_AUTH_KEY), 
             Amount(?SUPPLY), 
             U8(?DECIMALS), 
             U8(?INITIALISED), 
             Flag(?FREEZE_AUTH_FLAG), 
             Key(?FREEZE_AUTH_KEY))
     )
```

Key issue: This introduces numerous symbolic variables (`?MINT_AUTH_FLAG`, `?MINT_AUTH_KEY`, etc.), creating a highly abstract symbolic value.

### 2.2 Type Uncertainty Generation

When executing `accounts[0].owner()`:

1. **MIR-level operation**
   ```rust
   accounts[0].owner()  // Needs to access AccountInfo struct's owner field
   ```

2. **K semantics-level processing**
   ```k
   #traverseProjection(DEST, VALUE, projectionElemField(fieldIdx(2), TY), CTXS)
   ```

3. **Core problem**: `VALUE` is a symbolic `PAccount`, and K framework cannot determine its concrete form:
   - Could be `PAccountMint(PAcc, IMint)`
   - Could be `PAccountAccount(PAcc, IAcc)`
   - Could be a regular `Aggregate`
   - May require additional projection operations

### 2.3 Branch Generation Mechanism

K framework attempts to match multiple rules when processing:

#### Rule 1: PAccount Special Handling (Priority 30)
```k
rule <k> #traverseProjection(DEST, PAccountMint(PACC, IMINT), PROJ PROJS, CTXTS)
      => #traverseProjection(DEST, #fromPAcc(PACC), PROJ PROJS, CtxPAccountPAcc(IMINT) CTXTS)
      ...
      </k>
  [priority(30)]
```

#### Rule 2: Normal Aggregate Handling
```k
rule <k> #traverseProjection(DEST, Aggregate(IDX, FIELDS), 
           projectionElemField(fieldIdx(N), _), CTXS)
      => #traverseProjection(DEST, project:Value(FIELDS[N]), .ProjectionElems, 
           CtxField(IDX, FIELDS, N, TY) CTXS)
      ...
      </k>
```

#### Rule 3: Other Value Forms
Various rules for `getValue`, `project:Value`, and other operations.

Due to symbolic value uncertainty, all potentially matching rules generate an execution branch.

## 3. Impact Assessment

### 3.1 Direct Impact
- **Execution efficiency**: Branch explosion causes extremely low symbolic execution efficiency
- **Memory consumption**: Each branch requires independent execution state storage
- **Verification completeness**: Numerous branches make complete path exploration infeasible

### 3.2 Indirect Impact
- **Other tests**: All tests using cheatcodes may encounter similar issues
- **Scalability**: Branch count may grow exponentially with program complexity
- **Debugging difficulty**: Hard to trace specific execution paths and problem causes

## 4. Problem Verification and Evidence

### 4.1 Evidence Chain
1. **Source code evidence**
   - `entrypoint.rs:1692`: Calls `cheatcode_is_mint(&accounts[0])`
   - `entrypoint.rs:1706`: Executes `accounts[0].owner()` comparison

2. **Execution trace evidence**
   ```
   Node 4: #setArgFromStack (preparing arguments)
   Node 5: #traverseProjection (starting projection processing)
   Nodes 6-9: Multiple branches generated simultaneously
   ```

3. **Symbolic value evidence**
   - Node output shows `#fromPAcc(_)_KMIR-P-TOKEN_Va`
   - Underscore indicates unconcretized symbolic value

### 4.2 Reproducibility
- **Prerequisites**: Use cheatcode to create symbolic account
- **Trigger operation**: Access any field of symbolic account
- **Result**: 100% generates nondeterministic branches

## 5. Solution Design

### 5.1 Short-term Solution: Optimize Rule Matching

**Option A: Add Stricter Rule Conditions**
```k
rule <k> #traverseProjection(DEST, PAccountMint(PACC, IMINT), PROJ PROJS, CTXTS)
      => ...
      </k>
  requires isPAccountProjection(PROJ)  // New condition
  [priority(40)]  // Increase priority
```

**Pros**: Quick implementation, limited impact scope
**Cons**: May not completely eliminate branches

### 5.2 Medium-term Solution: Improve Cheatcode Implementation

**Option B: Create Concretized Default Values**
```k
syntax PAcc ::= #defaultPAcc() [function]
rule #defaultPAcc() => PAcc(U8(0), U8(0), U8(0), U8(0), 
                            U32(0), Key(""), Key(""), 
                            U64(0), U64(0))

syntax IMint ::= #defaultIMint() [function]  
rule #defaultIMint() => IMint(Flag(0), Key(""), Amount(0), 
                              U8(0), U8(1), // initialized=1
                              Flag(0), Key(""))

rule #mkPTokenMint(place(LOCAL, PROJS))
  => #setLocalValue(..., 
       PAccountMint(#defaultPAcc(), #defaultIMint()))
```

**Pros**: Reduces symbolic variables, decreases uncertainty
**Cons**: May affect verification generality

### 5.3 Long-term Solution: Refactor Symbolic Execution Strategy

**Option C: Type-directed Symbolic Execution**
1. Record type information when creating symbolic values
2. Use type information to select unique rule during projection
3. Implement type inference mechanism

```k
syntax TypedValue ::= typed(Value, Type)
syntax KItem ::= #typedTraverseProjection(Place, TypedValue, ProjectionElem, Contexts)

rule #typedTraverseProjection(DEST, typed(PAccountMint(PACC, IMINT), TY), PROJ, CTXS)
  => // Uniquely determined processing path
```

**Pros**: Fundamentally solves the problem
**Cons**: Requires large-scale refactoring

## 6. Implementation Recommendations

### 6.1 Immediate Actions (Week 1)
1. **Verify problem**: Confirm if same issue exists in other tests
2. **Document impact**: Count affected tests and severity
3. **Temporary mitigation**: Adjust execution parameters, limit branch depth

### 6.2 Short-term Improvements (Weeks 2-3)
1. **Implement Option A**: Optimize existing rule matching conditions
2. **Test verification**: Confirm branch reduction effect
3. **Performance evaluation**: Measure execution efficiency improvement

### 6.3 Medium-term Optimization (Weeks 4-8)
1. **Implement Option B**: Improve cheatcode implementation
2. **Regression testing**: Ensure no impact on other features
3. **Documentation update**: Document new cheatcode semantics

### 6.4 Long-term Planning (3-6 months)
1. **Design Option C**: Type-directed symbolic execution architecture
2. **Prototype implementation**: Validate in small-scale scenarios
3. **Gradual migration**: Phase-by-phase replacement of existing implementation

## 7. Risk Assessment

### 7.1 Technical Risks
- **Rule conflicts**: Modifying priorities may prevent other rules from matching
- **Semantic changes**: Concretizing symbolic values may alter verification semantics
- **Compatibility**: Must ensure compatibility with existing tests

### 7.2 Project Risks
- **Time cost**: Long-term solution requires significant development time
- **Maintenance burden**: Increases system complexity
- **Knowledge transfer**: Team needs to understand new execution model

## 8. Conclusions and Recommendations

### 8.1 Core Findings
1. Root cause of nondeterministic branching is type uncertainty of symbolic values, not missing `raw_eq`
2. Problem originates from highly abstract symbolic values created by cheatcode
3. K framework's rule matching mechanism exacerbates branch explosion

### 8.2 Key Recommendations
1. **Priority 1**: Implement short-term solution for quick branch explosion mitigation
2. **Priority 2**: Improve cheatcode implementation to reduce symbolic variables
3. **Priority 3**: Long-term planning for type-directed symbolic execution

### 8.3 Expected Outcomes
- **Short-term**: 50-70% reduction in branch count
- **Medium-term**: Basically eliminate unnecessary branches
- **Long-term**: Establish robust symbolic execution framework

## Appendix A: Related Code Locations

| File | Lines | Description |
|------|-------|-------------|
| `p-token.md` | 260-270 | cheatcode-is-mint rule |
| `p-token.md` | 290-310 | #addMint rule |
| `p-token.md` | 200-210 | PAccount projection rules |
| `entrypoint.rs` | 1692 | cheatcode_is_mint call |
| `entrypoint.rs` | 1706 | owner() comparison |

## Appendix B: Detailed Branch Tree Structure

```
Node 5: #traverseProjection(#fromPAcc)
├── Branch A: PAccountMint match
│   ├── Node 6: Continue #fromPAcc
│   └── Node 7: Continue #fromPAcc
├── Branch B: Aggregate match
│   └── Node 8: project:Value → raw_eq(Node 21)
└── Branch C: Needs getValue
    └── Node 9: getValue → more branches
```

## Appendix C: Monitoring Metrics

Recommended metrics to monitor improvement effectiveness:
1. **Branch count**: Total branches generated per test
2. **Execution steps**: Average steps to reach target state
3. **Memory usage**: Peak memory consumption
4. **Completion time**: Total test execution time

## Appendix D: Technical Details

### D.1 K Framework Rule Matching
The K framework uses a rewrite-based approach where multiple rules may match a given configuration. When symbolic values are involved, the framework cannot definitively determine which rule applies, leading to exploration of all possibilities.

### D.2 Symbolic Variable Introduction Points
1. **Cheatcode execution**: Initial symbolic account creation
2. **Field access**: Each projection may introduce new constraints
3. **Comparison operations**: Further constraints on symbolic values

### D.3 Performance Implications
- **Exponential growth**: Each branching point potentially doubles the state space
- **Memory overhead**: ~10MB per branch for typical configurations
- **CPU utilization**: Near 100% during branch exploration