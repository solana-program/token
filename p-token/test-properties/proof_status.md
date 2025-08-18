Analyzing proof files in: /home/zhaoji/solana-token/p-token/test-properties/artefacts/proof
Found 28 proof files

## entrypoint::test_process_amount_to_ui_amount - PASSED - 373
Node ├─ 6: <k>
│           #EndProgram ~> .K
│         </k>

Node └─ 2 (leaf, target, terminal): <k>
#EndProgram ~> .K
</k>

## entrypoint::test_process_approve - PASSED - 373
Node ├─ 6: <k>
│           #EndProgram ~> .K
│         </k>

Node └─ 2 (leaf, target, terminal): <k>
#EndProgram ~> .K
</k>

## entrypoint::test_process_approve_checked - PASSED - 373
Node ├─ 6: <k>
│           #EndProgram ~> .K
│         </k>

Node └─ 2 (leaf, target, terminal): <k>
#EndProgram ~> .K
</k>

## entrypoint::test_process_burn - Stuck - 605
Node └─ 9 (stuck, leaf): <k>
ListItem (thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 5 , place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ))
ListItem (Integer ( ARG_UINT71:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityMut ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindMut ( ... kind: mutBorrowKindDefault ) , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200305 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200306 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200301 ) ) )
</k>

## entrypoint::test_process_burn_checked - Stuck - 333
Node └─ 6 (stuck, leaf): <k>
ListItem (thunk ( #applyBinOp ( binOpOffset , PtrLocal ( 5 , place ( ... local: local ( 7 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , Integer ( 8 , 64 , false ) , false ) ))
ListItem (Integer ( 1 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 500031 ) , mutabilityNot ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 23 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 9 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindShared , place ( ... local: local ( 23 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 500049 ) )  statement ( ... kind: statementKindStorageDead ( local ( 23 ) ) , span: span ( 500050 ) )  statement ( ... kind: statementKindStorageDead ( local ( 12 ) ) , span: span ( 500059 ) )  statement ( ... kind: statementKindStorageDead ( local ( 10 ) ) , span: span ( 500059 ) )  statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueAggregate ( aggregateKindTuple , operandCopy ( place ( ... local: local ( 7 ) , projection: .ProjectionElems ) )  operandCopy ( place ( ... local: local ( 9 ) , projection: .ProjectionElems ) )  .Operands ) ) , span: span ( 500060 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 500058 ) ) )
</k>

## entrypoint::test_process_close_account - Unknown - 231
Node ├─ 5: <k>
│           #selectBlock ( switchTargets ( ... branches: branch ( 0 , basicBlockIdx ( 5 ) )  .Branches , otherwise: basicBlockIdx ( 4 ) ) , thunk ( #applyBinOp ( binOpEq , PtrLocal ( 4 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , PtrLocal ( 4 , place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , false ) ) ) ~> .K
│         </k>

Node └─ 6 (leaf, pending): <k>
┃              #execBlockIdx ( basicBlockIdx ( 5 ) ) ~> .K
┃            </k>

Node └─ 7 (leaf, pending): <k>
┃              #selectBlock ( switchTargets ( ... branches: .Branches , otherwise: basicBlockIdx ( 4 ) ) , thunk ( #applyBinOp ( binOpEq , PtrLocal ( 4 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , PtrLocal ( 4 , place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , false ) ) ) ~> .K
┃            </k>

Node └─ 8 (leaf, pending): <k>
#selectBlock ( switchTargets ( ... branches: branch ( 0 , basicBlockIdx ( 5 ) )  .Branches , otherwise: basicBlockIdx ( 4 ) ) , thunk ( #applyBinOp ( binOpEq , PtrLocal ( 4 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , PtrLocal ( 4 , place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , false ) ) ) ~> .K
</k>

## entrypoint::test_process_freeze_account - Stuck - 329
Node └─ 6 (stuck, leaf): <k>
ListItem (thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 5 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ))
ListItem (Integer ( ARG_UINT71:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityMut ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindMut ( ... kind: mutBorrowKindDefault ) , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200305 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200306 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200301 ) ) )
</k>

## entrypoint::test_process_get_account_data_size - Stuck - 224
Node └─ 5 (stuck, leaf): <k>
#execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 510347 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 500952 ) , id: mirConstId ( 882 ) ) ) ) , args: operandMove ( place ( ... local: local ( 1 ) , projection: .ProjectionElems ) )  operandMove ( place ( ... local: local ( 3 ) , projection: .ProjectionElems ) )  .Operands , destination: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionUnreachable ) , span: span ( 510348 ) ) ) ~> .K
</k>

## entrypoint::test_process_initialize_account - Stuck - 283
Node └─ 5 (stuck, leaf): <k>
ListItem (thunk ( #cast ( thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 4 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ) , castKindPtrToPtr , ty ( 500060 ) , ty ( 500002 ) ) ))
ListItem (Integer ( ARG_UINT71:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityNot ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindShared , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200258 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200259 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200254 ) ) )
</k>

## entrypoint::test_process_initialize_account2 - Stuck - 286
Node └─ 5 (stuck, leaf): <k>
ListItem (thunk ( #cast ( thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 4 , place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ) , castKindPtrToPtr , ty ( 500060 ) , ty ( 500002 ) ) ))
ListItem (Integer ( ARG_UINT71:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityNot ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindShared , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200258 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200259 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200254 ) ) )
</k>

## entrypoint::test_process_initialize_account3 - Stuck - 286
Node └─ 5 (stuck, leaf): <k>
ListItem (thunk ( #cast ( thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 4 , place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ) , castKindPtrToPtr , ty ( 500060 ) , ty ( 500002 ) ) ))
ListItem (Integer ( ARG_UINT71:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityNot ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindShared , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200258 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200259 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200254 ) ) )
</k>

## entrypoint::test_process_initialize_immutable_owner - Stuck - 497
Node └─ 7 (stuck, leaf): <k>
ListItem (thunk ( #cast ( thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 4 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ) , castKindPtrToPtr , ty ( 500060 ) , ty ( 500002 ) ) ))
ListItem (Integer ( ARG_UINT71:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityNot ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindShared , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200258 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200259 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200254 ) ) )
</k>

## entrypoint::test_process_initialize_mint2_freeze - Stuck - 118
Node └─ 4 (stuck, leaf): <k>
#execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 200211 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 200099 ) , id: mirConstId ( 29 ) ) ) ) , args: operandMove ( place ( ... local: local ( 1 ) , projection: .ProjectionElems ) )  .Operands , destination: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionUnreachable ) , span: span ( 200212 ) ) ) ~> .K
</k>

## entrypoint::test_process_initialize_mint2_no_freeze - Stuck - 118
Node └─ 4 (stuck, leaf): <k>
#execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 200211 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 200099 ) , id: mirConstId ( 29 ) ) ) ) , args: operandMove ( place ( ... local: local ( 1 ) , projection: .ProjectionElems ) )  .Operands , destination: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionUnreachable ) , span: span ( 200212 ) ) ) ~> .K
</k>

## entrypoint::test_process_initialize_mint_freeze - Stuck - 278
Node └─ 5 (stuck, leaf): <k>
ListItem (thunk ( #cast ( thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 3 , place ( ... local: local ( 4 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ) , castKindPtrToPtr , ty ( 500060 ) , ty ( 500002 ) ) ))
ListItem (Integer ( ARG_UINT142:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityNot ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindShared , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200258 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200259 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200254 ) ) )
</k>

## entrypoint::test_process_initialize_mint_no_freeze - Stuck - 278
Node └─ 5 (stuck, leaf): <k>
ListItem (thunk ( #cast ( thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 3 , place ( ... local: local ( 4 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ) , castKindPtrToPtr , ty ( 500060 ) , ty ( 500002 ) ) ))
ListItem (Integer ( ARG_UINT142:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityNot ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindShared , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200258 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200259 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200254 ) ) )
</k>

## entrypoint::test_process_initialize_multisig - Stuck - 649
Node └─ 9 (stuck, leaf): <k>
#traverseProjection ( toLocal ( 2 ) , thunk ( #decodeConstant ( constantKindAllocated ( allocation ( ... bytes: b"\x00\x00\x00\x00\x00\x00\x00\x00" , provenance: provenanceMap ( ... ptrs: provenanceMapEntry ( ... provSize: 0 , allocId: allocId ( 98 ) )  .ProvenanceMapEntries ) , align: align ( 8 ) , mutability: mutabilityMut ) ) , ty ( 500125 ) , typeInfoRefType ( ty ( 500016 ) ) ) ) , projectionElemDeref  .ProjectionElems , .Contexts )
~> #readProjection ( false )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 509992 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 500556 ) , id: mirConstId ( 432 ) ) ) ) , args: operandMove ( place ( ... local: local ( 3 ) , projection: .ProjectionElems ) )  operandMove ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) )  .Operands , destination: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionContinue ) , span: span ( 509993 ) ) )
</k>

## entrypoint::test_process_initialize_multisig2 - Stuck - 645
Node └─ 9 (stuck, leaf): <k>
#execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 200211 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 200099 ) , id: mirConstId ( 29 ) ) ) ) , args: operandMove ( place ( ... local: local ( 1 ) , projection: .ProjectionElems ) )  .Operands , destination: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionUnreachable ) , span: span ( 200212 ) ) ) ~> .K
</k>

## entrypoint::test_process_mint_to - Stuck - 114
Node └─ 4 (stuck, leaf): <k>
#traverseProjection ( toLocal ( 1 ) , Aggregate ( variantIdx ( 0 ) , ListItem (Reference ( 4 , place ( ... local: local ( 6 ) , projection: projectionElemConstantIndex ( ... offset: 0 , minLength: 0 , fromEnd: false )  .ProjectionElems ) , mutabilityNot , noMetadata ))
) , projectionElemDeref  projectionElemField ( fieldIdx ( 0 ) , ty ( 200144 ) )  .ProjectionElems , .Contexts )
~> #readProjection ( false )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , rvalue: rvalueCast ( castKindPtrToPtr , operandMove ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ) , ty ( 200180 ) ) ) , span: span ( 200754 ) )  statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , rvalue: rvalueCast ( castKindPtrToPtr , operandCopy ( place ( ... local: local ( 3 ) , projection: .ProjectionElems ) ) , ty ( 200040 ) ) ) , span: span ( 200755 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 200751 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 200179 ) , id: mirConstId ( 88 ) ) ) ) , args: .Operands , destination: place ( ... local: local ( 5 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionContinue ) , span: span ( 200752 ) ) )
</k>

## entrypoint::test_process_mint_to_checked - Stuck - 114
Node └─ 4 (stuck, leaf): <k>
#traverseProjection ( toLocal ( 1 ) , Aggregate ( variantIdx ( 0 ) , ListItem (Reference ( 4 , place ( ... local: local ( 6 ) , projection: projectionElemConstantIndex ( ... offset: 0 , minLength: 0 , fromEnd: false )  .ProjectionElems ) , mutabilityNot , noMetadata ))
) , projectionElemDeref  projectionElemField ( fieldIdx ( 0 ) , ty ( 200144 ) )  .ProjectionElems , .Contexts )
~> #readProjection ( false )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , rvalue: rvalueCast ( castKindPtrToPtr , operandMove ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ) , ty ( 200180 ) ) ) , span: span ( 200754 ) )  statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , rvalue: rvalueCast ( castKindPtrToPtr , operandCopy ( place ( ... local: local ( 3 ) , projection: .ProjectionElems ) ) , ty ( 200040 ) ) ) , span: span ( 200755 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 200751 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 200179 ) , id: mirConstId ( 88 ) ) ) ) , args: .Operands , destination: place ( ... local: local ( 5 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionContinue ) , span: span ( 200752 ) ) )
</k>

## entrypoint::test_process_revoke - Stuck - 292
Node └─ 5 (stuck, leaf): <k>
ListItem (thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 4 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ))
ListItem (Integer ( ARG_UINT71:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityMut ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindMut ( ... kind: mutBorrowKindDefault ) , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200305 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200306 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200301 ) ) )
</k>

## entrypoint::test_process_set_authority - PASSED - 210
Node ├─ 5: <k>
│           #EndProgram ~> .K
│         </k>

Node └─ 2 (leaf, target, terminal): <k>
#EndProgram ~> .K
</k>

## entrypoint::test_process_sync_native - Stuck - 416
Node └─ 7 (stuck, leaf): <k>
#execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 510347 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 500952 ) , id: mirConstId ( 882 ) ) ) ) , args: operandMove ( place ( ... local: local ( 1 ) , projection: .ProjectionElems ) )  operandMove ( place ( ... local: local ( 3 ) , projection: .ProjectionElems ) )  .Operands , destination: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionUnreachable ) , span: span ( 510348 ) ) ) ~> .K
</k>

## entrypoint::test_process_thaw_account - Stuck - 329
Node └─ 6 (stuck, leaf): <k>
ListItem (thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 5 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ))
ListItem (Integer ( ARG_UINT71:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityMut ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindMut ( ... kind: mutBorrowKindDefault ) , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200305 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200306 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200301 ) ) )
</k>

## entrypoint::test_process_transfer - Stuck - 208
Node └─ 5 (stuck, leaf): <k>
#traverseProjection ( toLocal ( 1 ) , Aggregate ( variantIdx ( 0 ) , ListItem (Reference ( 4 , place ( ... local: local ( 6 ) , projection: projectionElemConstantIndex ( ... offset: 0 , minLength: 0 , fromEnd: false )  .ProjectionElems ) , mutabilityNot , noMetadata ))
) , projectionElemDeref  projectionElemField ( fieldIdx ( 0 ) , ty ( 200144 ) )  .ProjectionElems , .Contexts )
~> #readProjection ( false )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , rvalue: rvalueCast ( castKindPtrToPtr , operandMove ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ) , ty ( 200180 ) ) ) , span: span ( 200754 ) )  statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , rvalue: rvalueCast ( castKindPtrToPtr , operandCopy ( place ( ... local: local ( 3 ) , projection: .ProjectionElems ) ) , ty ( 200040 ) ) ) , span: span ( 200755 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 200751 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 200179 ) , id: mirConstId ( 88 ) ) ) ) , args: .Operands , destination: place ( ... local: local ( 5 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionContinue ) , span: span ( 200752 ) ) )
</k>

## entrypoint::test_process_transfer_checked - Stuck - 208
Node └─ 5 (stuck, leaf): <k>
#traverseProjection ( toLocal ( 1 ) , Aggregate ( variantIdx ( 0 ) , ListItem (Reference ( 4 , place ( ... local: local ( 7 ) , projection: projectionElemConstantIndex ( ... offset: 0 , minLength: 0 , fromEnd: false )  .ProjectionElems ) , mutabilityNot , noMetadata ))
) , projectionElemDeref  projectionElemField ( fieldIdx ( 0 ) , ty ( 200144 ) )  .ProjectionElems , .Contexts )
~> #readProjection ( false )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 3 ) , projection: .ProjectionElems ) , rvalue: rvalueCast ( castKindPtrToPtr , operandMove ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ) , ty ( 200180 ) ) ) , span: span ( 200754 ) )  statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , rvalue: rvalueCast ( castKindPtrToPtr , operandCopy ( place ( ... local: local ( 3 ) , projection: .ProjectionElems ) ) , ty ( 200040 ) ) ) , span: span ( 200755 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 200751 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 200179 ) , id: mirConstId ( 88 ) ) ) ) , args: .Operands , destination: place ( ... local: local ( 5 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionContinue ) , span: span ( 200752 ) ) )
</k>

## entrypoint::test_process_ui_amount_to_amount - Stuck - 44
Node └─ 3 (stuck, leaf): <k>
#execTerminator ( terminator ( ... kind: terminatorKindCall ( ... func: operandConstant ( constOperand ( ... span: span ( 501615 ) , userTy: noUserTypeAnnotationIndex , const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 500385 ) , id: mirConstId ( 263 ) ) ) ) , args: operandCopy ( place ( ... local: local ( 2 ) , projection: .ProjectionElems ) )  .Operands , destination: place ( ... local: local ( 5 ) , projection: .ProjectionElems ) , target: someBasicBlockIdx ( basicBlockIdx ( 1 ) ) , unwind: unwindActionContinue ) , span: span ( 501616 ) ) ) ~> .K
</k>

## entrypoint::test_process_withdraw_excess_lamports - Stuck - 327
Node └─ 6 (stuck, leaf): <k>
ListItem (thunk ( #cast ( thunk ( #applyBinOp ( binOpOffset , thunk ( #cast ( PtrLocal ( 4 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , ptrEmulation ( noMetadata ) ) , castKindPtrToPtr , ty ( 200180 ) , ty ( 200040 ) ) ) , thunk ( rvalueNullaryOp ( nullOpSizeOf , ty ( 200075 ) ) ) , false ) ) , castKindPtrToPtr , ty ( 500060 ) , ty ( 500002 ) ) ))
ListItem (Integer ( ARG_UINT71:Int +Int 18446744073709551616 %Int 18446744073709551616 , 64 , false ))
~> #mkAggregate ( aggregateKindRawPtr ( ty ( 200113 ) , mutabilityNot ) )
~> #freezer#setLocalValue(_,_)_RT-DATA_KItem_Place_Evaluation1_ ( place ( ... local: local ( 8 ) , projection: .ProjectionElems ) ~> .K )
~> #execStmts ( statement ( ... kind: statementKindAssign ( ... place: place ( ... local: local ( 0 ) , projection: .ProjectionElems ) , rvalue: rvalueRef ( region ( ... kind: regionKindReErased ) , borrowKindShared , place ( ... local: local ( 8 ) , projection: projectionElemDeref  .ProjectionElems ) ) ) , span: span ( 200258 ) )  statement ( ... kind: statementKindStorageDead ( local ( 8 ) ) , span: span ( 200259 ) )  .Statements )
~> #execTerminator ( terminator ( ... kind: terminatorKindReturn , span: span ( 200254 ) ) )
</k>

