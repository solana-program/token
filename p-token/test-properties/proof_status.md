# Proof test_process_amount_to_ui_amount                            

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   281
├─ 4     NORMAL                        │ line  4711
├─ 5  🔀 SPLIT     ((split))            │ line  9185
┃  ├─ 6     NORMAL                        │ line 13752
┃  ├─ 8     NORMAL                        │ line 18316
┃  ├─ 10    NORMAL                        │ line 22792
┃  └─ 13 ❌ STUCK     ((stuck, leaf))      │ line 27271
   ├─ 7     NORMAL                        │ line 31742
   ├─ 9  🔀 SPLIT     ((split))            │ line 36273
   ┃  ├─ 11    NORMAL                        │ line 40706
   ┃  ├─ 14    NORMAL                        │ line 45136
   ┃  ├─ 16    NORMAL                        │ line 49556
   ┃  ├─ 18    NORMAL                        │ line 53968
   ┃  ├─ 20    NORMAL                        │ line 58379
   ┃  └─ 21 ❌ STUCK     ((stuck, leaf))      │ line 62716
      ├─ 12    NORMAL                        │ line 67092
      ├─ 15    NORMAL                        │ line 71522
      ├─ 17    NORMAL                        │ line 75932
      └─ 19 ❌ STUCK     ((stuck, leaf))      │ line 80643
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 85387

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 21

  ⚪ Normal      :  14
  🌳 Root        :   2
  🔀 Split       :   2
  ❌ Stuck       :   3

🎯 Proof Outcome:
  ❌ STUCK: 3 branch(es) got stuck

```

# Proof test_process_approve_checked                                

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   941
├─ 4     NORMAL                        │ line  6554
└─ 5  ❌ STUCK     ((stuck, leaf))      │ line 12185
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 17507

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 5

  ⚪ Normal      :   2
  🌳 Root        :   2
  ❌ Stuck       :   1

🎯 Proof Outcome:
  ❌ STUCK: 1 branch(es) got stuck

```

# Proof test_process_approve                                        

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   719
├─ 4     NORMAL                        │ line  5962
└─ 5  ❌ STUCK     ((stuck, leaf))      │ line 10984
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 16002

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 5

  ⚪ Normal      :   2
  🌳 Root        :   2
  ❌ Stuck       :   1

🎯 Proof Outcome:
  ❌ STUCK: 1 branch(es) got stuck

```

# Proof test_process_burn_checked                                   

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   722
├─ 4  🔀 SPLIT     ((split))            │ line  6584
┃  ├─ 5     NORMAL                        │ line 11910
┃  ├─ 7     NORMAL                        │ line 17233
┃  ├─ 9     NORMAL                        │ line 23110
┃  ├─ 10    NORMAL                        │ line 28988
┃  ├─ 11 🔀 SPLIT     ((split))            │ line 34330
┃  ┃  ├─ 12    NORMAL                        │ line 39727
┃  ┃  ├─ 14    NORMAL                        │ line 45121
┃  ┃  ├─ 16 🔀 SPLIT     ((split))            │ line 51064
┃  ┃  ┃  ├─ 18    NORMAL                        │ line 56462
┃  ┃  ┃  ├─ 22    NORMAL                        │ line 61857
┃  ┃  ┃  ┃  └─ 26 ⏳ PENDING   ((leaf, pending))    │ line 67379
┃  ┃  ┃  ┃  └─ 27 ⏳ PENDING   ((leaf, pending))    │ line 72901
┃  ┃  ┃     └─ 28 ⏳ PENDING   ((leaf, pending))    │ line 78424
┃  ┃     ├─ 19    NORMAL                        │ line 83951
┃  ┃     └─ 23 ⏳ PENDING   ((leaf, pending))    │ line 89346
┃     ├─ 13    NORMAL                        │ line 95332
┃     ├─ 15    NORMAL                        │ line 100726
┃     ├─ 17 🔀 SPLIT     ((split))            │ line 106158
┃     ┃  ├─ 20    NORMAL                        │ line 111586
┃     ┃  └─ 24 ⏳ PENDING   ((leaf, pending))    │ line 117011
┃        ├─ 21    NORMAL                        │ line 122596
┃        └─ 25 ⏳ PENDING   ((leaf, pending))    │ line 128021
   ├─ 6     NORMAL                        │ line 134006
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 139329
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 144682

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 28

  ⚪ Normal      :  15
  ⏳ Pending     :   6
  🌳 Root        :   2
  🔀 Split       :   4
  ❌ Stuck       :   1

🎯 Proof Outcome:
  ❌ STUCK: 1 branch(es) got stuck
  ⏳ PENDING: 6 branch(es) still pending

```

# Proof test_process_burn                                           

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   719
├─ 4  🔀 SPLIT     ((split))            │ line  6538
┃  ├─ 5     NORMAL                        │ line 11841
┃  ├─ 7     NORMAL                        │ line 17141
┃  ├─ 9     NORMAL                        │ line 22975
┃  ├─ 10    NORMAL                        │ line 28810
┃  ├─ 11 🔀 SPLIT     ((split))            │ line 34129
┃  ┃  ├─ 12    NORMAL                        │ line 39503
┃  ┃  ├─ 14    NORMAL                        │ line 44874
┃  ┃  ├─ 16 🔀 SPLIT     ((split))            │ line 50774
┃  ┃  ┃  ├─ 18    NORMAL                        │ line 56149
┃  ┃  ┃  ├─ 22    NORMAL                        │ line 61521
┃  ┃  ┃  ┃  └─ 26 ⏳ PENDING   ((leaf, pending))    │ line 67020
┃  ┃  ┃  ┃  └─ 27 ⏳ PENDING   ((leaf, pending))    │ line 72519
┃  ┃  ┃     └─ 28 ⏳ PENDING   ((leaf, pending))    │ line 78019
┃  ┃     ├─ 19    NORMAL                        │ line 83523
┃  ┃     └─ 23 ⏳ PENDING   ((leaf, pending))    │ line 88895
┃     ├─ 13    NORMAL                        │ line 94838
┃     ├─ 15    NORMAL                        │ line 100209
┃     ├─ 17 🔀 SPLIT     ((split))            │ line 105618
┃     ┃  ├─ 20    NORMAL                        │ line 111023
┃     ┃  └─ 24 ⏳ PENDING   ((leaf, pending))    │ line 116425
┃        ├─ 21    NORMAL                        │ line 121987
┃        └─ 25 ⏳ PENDING   ((leaf, pending))    │ line 127389
   ├─ 6     NORMAL                        │ line 133331
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 138631
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 143961

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 28

  ⚪ Normal      :  15
  ⏳ Pending     :   6
  🌳 Root        :   2
  🔀 Split       :   4
  ❌ Stuck       :   1

🎯 Proof Outcome:
  ❌ STUCK: 1 branch(es) got stuck
  ⏳ PENDING: 6 branch(es) still pending

```

# Proof test_process_close_account                                  

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   691
├─ 4  🔀 SPLIT     ((split))            │ line  5713
┃  ├─ 5     NORMAL                        │ line 10747
┃  ├─ 7     NORMAL                        │ line 15778
┃  ├─ 9     NORMAL                        │ line 20801
┃  ├─ 10    NORMAL                        │ line 26125
┃  ├─ 11 🔀 SPLIT     ((split))            │ line 31171
┃  ┃  ├─ 12    NORMAL                        │ line 36275
┃  ┃  ├─ 14    NORMAL                        │ line 41376
┃  ┃  ├─ 16 🔀 SPLIT     ((split))            │ line 46478
┃  ┃  ┃  ├─ 19    NORMAL                        │ line 51583
┃  ┃  ┃  ├─ 23    NORMAL                        │ line 56685
┃  ┃  ┃  └─ 27 ❌ STUCK     ((stuck, leaf))      │ line 61785
┃  ┃     ├─ 20    NORMAL                        │ line 66896
┃  ┃     ├─ 24    NORMAL                        │ line 71998
┃  ┃     └─ 28 ❌ STUCK     ((stuck, leaf))      │ line 77389
┃     ├─ 13    NORMAL                        │ line 82533
┃     ├─ 15 🔀 SPLIT     ((split))            │ line 87634
┃     ┃  ├─ 17    NORMAL                        │ line 92737
┃     ┃  ├─ 21    NORMAL                        │ line 97837
┃     ┃  └─ 25 ❌ STUCK     ((stuck, leaf))      │ line 102967
┃        ├─ 18    NORMAL                        │ line 108076
┃        ├─ 22    NORMAL                        │ line 113176
┃        └─ 26 ❌ STUCK     ((stuck, leaf))      │ line 118597
   ├─ 6     NORMAL                        │ line 123740
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 128771
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 133832

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 28

  ⚪ Normal      :  17
  🌳 Root        :   2
  🔀 Split       :   4
  ❌ Stuck       :   5

🎯 Proof Outcome:
  ❌ STUCK: 5 branch(es) got stuck

```

# Proof test_process_freeze_account                                 

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   691
├─ 4  🔀 SPLIT     ((split))            │ line  5710
┃  ├─ 5     NORMAL                        │ line 10741
┃  ├─ 7     NORMAL                        │ line 15769
┃  ├─ 9     NORMAL                        │ line 20799
┃  ├─ 10 🔀 SPLIT     ((split))            │ line 26115
┃  ┃  ├─ 11    NORMAL                        │ line 31137
┃  ┃  ├─ 13 🔀 SPLIT     ((split))            │ line 36156
┃  ┃  ┃  ├─ 15    NORMAL                        │ line 41186
┃  ┃  ┃  ├─ 19    NORMAL                        │ line 46213
┃  ┃  ┃  └─ 23 ❌ STUCK     ((stuck, leaf))      │ line 51235
┃  ┃     ├─ 16    NORMAL                        │ line 56272
┃  ┃     └─ 20 ❌ STUCK     ((stuck, leaf))      │ line 61299
┃     ├─ 12    NORMAL                        │ line 66337
┃     ├─ 14 🔀 SPLIT     ((split))            │ line 71356
┃     ┃  ├─ 17    NORMAL                        │ line 76386
┃     ┃  ├─ 21    NORMAL                        │ line 81413
┃     ┃  └─ 24 ❌ STUCK     ((stuck, leaf))      │ line 86435
┃        ├─ 18    NORMAL                        │ line 91472
┃        └─ 22 ❌ STUCK     ((stuck, leaf))      │ line 96499
   ├─ 6     NORMAL                        │ line 101537
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 106565
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 111623

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 24

  ⚪ Normal      :  13
  🌳 Root        :   2
  🔀 Split       :   4
  ❌ Stuck       :   5

🎯 Proof Outcome:
  ❌ STUCK: 5 branch(es) got stuck

```

# Proof test_process_get_account_data_size                          

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   253
├─ 4  🔀 SPLIT     ((split))            │ line  4633
┃  ├─ 5     NORMAL                        │ line  9117
┃  ├─ 7     NORMAL                        │ line 13598
┃  ├─ 9     NORMAL                        │ line 18021
┃  └─ 12 ❌ STUCK     ((stuck, leaf))      │ line 22415
   ├─ 6     NORMAL                        │ line 26846
   ├─ 8  🔀 SPLIT     ((split))            │ line 31294
   ┃  ├─ 10    NORMAL                        │ line 35644
   ┃  ├─ 13    NORMAL                        │ line 39991
   ┃  ├─ 15    NORMAL                        │ line 44328
   ┃  ├─ 17    NORMAL                        │ line 48686
   ┃  ├─ 19    NORMAL                        │ line 53045
   ┃  └─ 21 ❌ STUCK     ((stuck, leaf))      │ line 57351
      ├─ 11    NORMAL                        │ line 61687
      ├─ 14    NORMAL                        │ line 66034
      ├─ 16    NORMAL                        │ line 70361
      ├─ 18    NORMAL                        │ line 74718
      ├─ 20 ✅ TERMINAL  ((terminal))         │ line 79074
      └─ 2  ✅ TERMINAL  ((leaf, target, terminal)) │ line 83432

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 21

  ⚪ Normal      :  14
  🌳 Root        :   1
  🔀 Split       :   2
  ❌ Stuck       :   2
  ✅ Terminal    :   2

🎯 Proof Outcome:
  ✅ SUCCESS: Found 2 terminal state(s)
  ❌ STUCK: 2 branch(es) got stuck

```

# Proof test_process_initialize_account2                            

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   791
├─ 4  🔀 SPLIT     ((split))            │ line  6113
┃  ├─ 5     NORMAL                        │ line 11188
┃  └─ 7  ❌ STUCK     ((stuck, leaf))      │ line 16260
   ├─ 6     NORMAL                        │ line 21334
   ├─ 8     NORMAL                        │ line 26406
   ├─ 9  🔀 SPLIT     ((split))            │ line 31503
   ┃  ├─ 10    NORMAL                        │ line 36600
   ┃  ├─ 12 🔀 SPLIT     ((split))            │ line 41694
   ┃  ┃  ├─ 14    NORMAL                        │ line 46797
   ┃  ┃  └─ 18 ❌ STUCK     ((stuck, leaf))      │ line 51897
   ┃     ├─ 15    NORMAL                        │ line 57000
   ┃     ├─ 19    NORMAL                        │ line 62100
   ┃     ├─ 22 🔀 SPLIT     ((split))            │ line 67488
   ┃     ┃  ├─ 25    NORMAL                        │ line 72619
   ┃     ┃  ├─ 29    NORMAL                        │ line 77747
   ┃     ┃  ├─ 32    NORMAL                        │ line 83069
   ┃     ┃  ├─ 36 🔀 SPLIT     ((split))            │ line 88319
   ┃     ┃  ┃  └─ 40 ⏳ PENDING   ((leaf, pending))    │ line 93669
   ┃     ┃     └─ 41 ⏳ PENDING   ((leaf, pending))    │ line 99083
   ┃        ├─ 26    NORMAL                        │ line 104466
   ┃        ├─ 30    NORMAL                        │ line 109594
   ┃        ├─ 33    NORMAL                        │ line 114916
   ┃        ├─ 37 🔀 SPLIT     ((split))            │ line 120166
   ┃        ┃  └─ 42 ⏳ PENDING   ((leaf, pending))    │ line 125516
   ┃           └─ 43 ⏳ PENDING   ((leaf, pending))    │ line 130930
      ├─ 11    NORMAL                        │ line 136313
      ├─ 13 🔀 SPLIT     ((split))            │ line 141407
      ┃  ├─ 16    NORMAL                        │ line 146510
      ┃  └─ 20 ❌ STUCK     ((stuck, leaf))      │ line 151610
         ├─ 17    NORMAL                        │ line 156713
         ├─ 21 🔀 SPLIT     ((split))            │ line 161813
         ┃  ├─ 23    NORMAL                        │ line 166920
         ┃  └─ 27 ❌ STUCK     ((stuck, leaf))      │ line 172024
            ├─ 24    NORMAL                        │ line 177131
            ├─ 28    NORMAL                        │ line 182235
            ├─ 31 🔀 SPLIT     ((split))            │ line 187368
            ┃  ├─ 34    NORMAL                        │ line 192500
            ┃  └─ 38 ⏳ PENDING   ((leaf, pending))    │ line 197629
               ├─ 35    NORMAL                        │ line 202955
               └─ 39 ⏳ PENDING   ((leaf, pending))    │ line 208084
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 213407

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 43

  ⚪ Normal      :  22
  ⏳ Pending     :   6
  🌳 Root        :   2
  🔀 Split       :   9
  ❌ Stuck       :   4

🎯 Proof Outcome:
  ❌ STUCK: 4 branch(es) got stuck
  ⏳ PENDING: 6 branch(es) still pending

```

# Proof test_process_initialize_account3                            

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   572
├─ 4  🔀 SPLIT     ((split))            │ line  5399
┃  ├─ 5     NORMAL                        │ line 10238
┃  └─ 7  ❌ STUCK     ((stuck, leaf))      │ line 15074
   ├─ 6     NORMAL                        │ line 19912
   ├─ 8     NORMAL                        │ line 24748
   ┃  ├─ 9     NORMAL                        │ line 29592
   ┃  └─ 12 ❌ STUCK     ((stuck, leaf))      │ line 34435
   ┃  ├─ 10    NORMAL                        │ line 39279
   ┃  └─ 13 🍃 LEAF      ((vacuous, leaf))    │ line 44122
      └─ 11 ❌ STUCK     ((stuck, leaf))      │ line 49693
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 54537

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 13

  🍃 Leaf        :   1
  ⚪ Normal      :   6
  🌳 Root        :   2
  🔀 Split       :   1
  ❌ Stuck       :   3

🎯 Proof Outcome:
  ❌ STUCK: 3 branch(es) got stuck

```

# Proof test_process_initialize_account                             

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   910
├─ 4  🔀 SPLIT     ((split))            │ line  6358
┃  ├─ 5     NORMAL                        │ line 11559
┃  └─ 7  ❌ STUCK     ((stuck, leaf))      │ line 16757
   ├─ 6     NORMAL                        │ line 21957
   ├─ 8     NORMAL                        │ line 27155
   ├─ 9  🔀 SPLIT     ((split))            │ line 32378
   ┃  ├─ 10    NORMAL                        │ line 37601
   ┃  ├─ 12 🔀 SPLIT     ((split))            │ line 42821
   ┃  ┃  ├─ 14    NORMAL                        │ line 48050
   ┃  ┃  └─ 18 ❌ STUCK     ((stuck, leaf))      │ line 53276
   ┃     ├─ 15    NORMAL                        │ line 58505
   ┃     ├─ 19    NORMAL                        │ line 63731
   ┃     ├─ 22 🔀 SPLIT     ((split))            │ line 69241
   ┃     ┃  ├─ 25    NORMAL                        │ line 74498
   ┃     ┃  ├─ 29    NORMAL                        │ line 79752
   ┃     ┃  ├─ 32    NORMAL                        │ line 85123
   ┃     ┃  ├─ 36 🔀 SPLIT     ((split))            │ line 90568
   ┃     ┃  ┃  └─ 40 ⏳ PENDING   ((leaf, pending))    │ line 96035
   ┃     ┃     └─ 41 ⏳ PENDING   ((leaf, pending))    │ line 101566
   ┃        ├─ 26    NORMAL                        │ line 107066
   ┃        ├─ 30    NORMAL                        │ line 112320
   ┃        ├─ 33    NORMAL                        │ line 117682
   ┃        ├─ 37 🔀 SPLIT     ((split))            │ line 123127
   ┃        ┃  └─ 42 ⏳ PENDING   ((leaf, pending))    │ line 128594
   ┃           └─ 43 ⏳ PENDING   ((leaf, pending))    │ line 134125
      ├─ 11    NORMAL                        │ line 139625
      ├─ 13 🔀 SPLIT     ((split))            │ line 144845
      ┃  ├─ 16    NORMAL                        │ line 150074
      ┃  └─ 20 ❌ STUCK     ((stuck, leaf))      │ line 155300
         ├─ 17    NORMAL                        │ line 160529
         ├─ 21 🔀 SPLIT     ((split))            │ line 165755
         ┃  ├─ 23    NORMAL                        │ line 170988
         ┃  └─ 27 ❌ STUCK     ((stuck, leaf))      │ line 176218
            ├─ 24    NORMAL                        │ line 181451
            ├─ 28    NORMAL                        │ line 186681
            ├─ 31 🔀 SPLIT     ((split))            │ line 191940
            ┃  ├─ 34    NORMAL                        │ line 197198
            ┃  └─ 38 ⏳ PENDING   ((leaf, pending))    │ line 202453
               ├─ 35    NORMAL                        │ line 207828
               └─ 39 ⏳ PENDING   ((leaf, pending))    │ line 213083
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 218446

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 43

  ⚪ Normal      :  22
  ⏳ Pending     :   6
  🌳 Root        :   2
  🔀 Split       :   9
  ❌ Stuck       :   4

🎯 Proof Outcome:
  ❌ STUCK: 4 branch(es) got stuck
  ⏳ PENDING: 6 branch(es) still pending

```

# Proof test_process_initialize_immutable_owner                     

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   253
├─ 4  🔀 SPLIT     ((split))            │ line  4581
┃  ├─ 5     NORMAL                        │ line  8909
┃  ├─ 7     NORMAL                        │ line 13234
┃  ├─ 9     NORMAL                        │ line 17582
┃  ├─ 10    NORMAL                        │ line 21937
┃  └─ 11 ❌ STUCK     ((stuck, leaf))      │ line 26297
   ├─ 6     NORMAL                        │ line 30624
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 34949
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 39304

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 11

  ⚪ Normal      :   6
  🌳 Root        :   2
  🔀 Split       :   1
  ❌ Stuck       :   2

🎯 Proof Outcome:
  ❌ STUCK: 2 branch(es) got stuck

```

# Proof test_process_initialize_mint2_freeze                        

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   455
┃  ├─ 4     NORMAL                        │ line  5118
┃  └─ 7  ❌ STUCK     ((stuck, leaf))      │ line  9780
┃  ├─ 5     NORMAL                        │ line 14443
┃  └─ 8  🍃 LEAF      ((vacuous, leaf))    │ line 19105
   └─ 6  ❌ STUCK     ((stuck, leaf))      │ line 24333
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 28996

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 8

  🍃 Leaf        :   1
  ⚪ Normal      :   3
  🌳 Root        :   2
  ❌ Stuck       :   2

🎯 Proof Outcome:
  ❌ STUCK: 2 branch(es) got stuck

```

# Proof test_process_initialize_mint2_no_freeze                     

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   359
┃  ├─ 4     NORMAL                        │ line  4926
┃  └─ 7  ❌ STUCK     ((stuck, leaf))      │ line  9492
┃  ├─ 5     NORMAL                        │ line 14059
┃  └─ 8  🍃 LEAF      ((vacuous, leaf))    │ line 18625
   └─ 6  ❌ STUCK     ((stuck, leaf))      │ line 23757
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 28324

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 8

  🍃 Leaf        :   1
  ⚪ Normal      :   3
  🌳 Root        :   2
  ❌ Stuck       :   2

🎯 Proof Outcome:
  ❌ STUCK: 2 branch(es) got stuck

```

# Proof test_process_initialize_mint_freeze                         

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   674
├─ 4  🔀 SPLIT     ((split))            │ line  5726
┃  ├─ 5     NORMAL                        │ line 10625
┃  ├─ 7  🔀 SPLIT     ((split))            │ line 15521
┃  ┃  ├─ 9     NORMAL                        │ line 20426
┃  ┃  └─ 13 ❌ STUCK     ((stuck, leaf))      │ line 25328
┃     ├─ 10    NORMAL                        │ line 30233
┃     ├─ 14 🔀 SPLIT     ((split))            │ line 35135
┃     ┃  ├─ 17    NORMAL                        │ line 40004
┃     ┃  └─ 21 ❌ STUCK     ((stuck, leaf))      │ line 44870
┃        ├─ 18    NORMAL                        │ line 49842
┃        └─ 22 ❌ STUCK     ((stuck, leaf))      │ line 54708
   ├─ 6     NORMAL                        │ line 59680
   ├─ 8  🔀 SPLIT     ((split))            │ line 64576
   ┃  ├─ 11    NORMAL                        │ line 69481
   ┃  └─ 15 ❌ STUCK     ((stuck, leaf))      │ line 74383
      ├─ 12    NORMAL                        │ line 79288
      ├─ 16 🔀 SPLIT     ((split))            │ line 84190
      ┃  ├─ 19    NORMAL                        │ line 89099
      ┃  └─ 23 ❌ STUCK     ((stuck, leaf))      │ line 94005
         ├─ 20    NORMAL                        │ line 98914
         ├─ 24 🔀 SPLIT     ((split))            │ line 103820
         ┃  ├─ 25    NORMAL                        │ line 108690
         ┃  └─ 27 ❌ STUCK     ((stuck, leaf))      │ line 113557
            ├─ 26    NORMAL                        │ line 118530
            └─ 28 ❌ STUCK     ((stuck, leaf))      │ line 123397
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 128367

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 28

  ⚪ Normal      :  13
  🌳 Root        :   2
  🔀 Split       :   6
  ❌ Stuck       :   7

🎯 Proof Outcome:
  ❌ STUCK: 7 branch(es) got stuck

```

# Proof test_process_initialize_mint_no_freeze                      

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   578
├─ 4  🔀 SPLIT     ((split))            │ line  5534
┃  ├─ 5     NORMAL                        │ line 10337
┃  ├─ 7  🔀 SPLIT     ((split))            │ line 15137
┃  ┃  ├─ 9     NORMAL                        │ line 19946
┃  ┃  └─ 13 ❌ STUCK     ((stuck, leaf))      │ line 24752
┃     ├─ 10    NORMAL                        │ line 29561
┃     ├─ 14 🔀 SPLIT     ((split))            │ line 34367
┃     ┃  ├─ 17    NORMAL                        │ line 39140
┃     ┃  └─ 21 ❌ STUCK     ((stuck, leaf))      │ line 43910
┃        ├─ 18    NORMAL                        │ line 48786
┃        └─ 22 ❌ STUCK     ((stuck, leaf))      │ line 53556
   ├─ 6     NORMAL                        │ line 58432
   ├─ 8  🔀 SPLIT     ((split))            │ line 63232
   ┃  ├─ 11    NORMAL                        │ line 68041
   ┃  └─ 15 ❌ STUCK     ((stuck, leaf))      │ line 72847
      ├─ 12    NORMAL                        │ line 77656
      ├─ 16 🔀 SPLIT     ((split))            │ line 82462
      ┃  ├─ 19    NORMAL                        │ line 87275
      ┃  └─ 23 ❌ STUCK     ((stuck, leaf))      │ line 92085
         ├─ 20    NORMAL                        │ line 96898
         ├─ 24 🔀 SPLIT     ((split))            │ line 101708
         ┃  ├─ 25    NORMAL                        │ line 106482
         ┃  └─ 27 ❌ STUCK     ((stuck, leaf))      │ line 111253
            ├─ 26    NORMAL                        │ line 116130
            └─ 28 ❌ STUCK     ((stuck, leaf))      │ line 120901
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 125775

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 28

  ⚪ Normal      :  13
  🌳 Root        :   2
  🔀 Split       :   6
  ❌ Stuck       :   7

🎯 Proof Outcome:
  ❌ STUCK: 7 branch(es) got stuck

```

# Proof test_process_mint_to_checked                                

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   722
├─ 4     NORMAL                        │ line  6291
└─ 5  ❌ STUCK     ((stuck, leaf))      │ line 11469
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 16658

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 5

  ⚪ Normal      :   2
  🌳 Root        :   2
  ❌ Stuck       :   1

🎯 Proof Outcome:
  ❌ STUCK: 1 branch(es) got stuck

```

# Proof test_process_mint_to                                        

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   719
├─ 4     NORMAL                        │ line  6245
└─ 5  ❌ STUCK     ((stuck, leaf))      │ line 11400
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 16566

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 5

  ⚪ Normal      :   2
  🌳 Root        :   2
  ❌ Stuck       :   1

🎯 Proof Outcome:
  ❌ STUCK: 1 branch(es) got stuck

```

# Proof test_process_revoke                                         

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   472
├─ 4  🔀 SPLIT     ((split))            │ line  5430
┃  ├─ 5     NORMAL                        │ line 10173
┃  ├─ 7     NORMAL                        │ line 14913
┃  ├─ 9     NORMAL                        │ line 19655
┃  └─ 10 ❌ STUCK     ((stuck, leaf))      │ line 24404
   ├─ 6     NORMAL                        │ line 29153
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 33893
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 38663

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 10

  ⚪ Normal      :   5
  🌳 Root        :   2
  🔀 Split       :   1
  ❌ Stuck       :   2

🎯 Proof Outcome:
  ❌ STUCK: 2 branch(es) got stuck

```

# Proof test_process_sync_native                                    

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   253
├─ 4  🔀 SPLIT     ((split))            │ line  4675
┃  ├─ 5     NORMAL                        │ line  9092
┃  ├─ 7     NORMAL                        │ line 13506
┃  ├─ 9  🔀 SPLIT     ((split))            │ line 17924
┃  ┃  ├─ 10    NORMAL                        │ line 22335
┃  ┃  ├─ 12    NORMAL                        │ line 26743
┃  ┃  ├─ 14    NORMAL                        │ line 31290
┃  ┃  ├─ 16 🔀 SPLIT     ((split))            │ line 35758
┃  ┃  ┃  ├─ 18    NORMAL                        │ line 40348
┃  ┃  ┃  ├─ 22    NORMAL                        │ line 44935
┃  ┃  ┃  └─ 26 ❌ STUCK     ((stuck, leaf))      │ line 49387
┃  ┃     ├─ 19    NORMAL                        │ line 53906
┃  ┃     ├─ 23    NORMAL                        │ line 58460
┃  ┃     ├─ 27    NORMAL                        │ line 62927
┃  ┃     ├─ 30    NORMAL                        │ line 67387
┃  ┃     ├─ 32    NORMAL                        │ line 71777
┃  ┃     └─ 34 ❌ STUCK     ((stuck, leaf))      │ line 76171
┃     ├─ 11    NORMAL                        │ line 80564
┃     ├─ 13    NORMAL                        │ line 84972
┃     ├─ 15    NORMAL                        │ line 89384
┃     ├─ 17 🔀 SPLIT     ((split))            │ line 93878
┃     ┃  ├─ 20    NORMAL                        │ line 98475
┃     ┃  ├─ 24    NORMAL                        │ line 103069
┃     ┃  └─ 28 ❌ STUCK     ((stuck, leaf))      │ line 107528
┃        ├─ 21    NORMAL                        │ line 112054
┃        ├─ 25    NORMAL                        │ line 116615
┃        ├─ 29    NORMAL                        │ line 121089
┃        ├─ 31    NORMAL                        │ line 125556
┃        ├─ 33    NORMAL                        │ line 129953
┃        └─ 35 ❌ STUCK     ((stuck, leaf))      │ line 134354
   ├─ 6     NORMAL                        │ line 138754
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 143168
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 147612

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 35

  ⚪ Normal      :  24
  🌳 Root        :   2
  🔀 Split       :   4
  ❌ Stuck       :   5

🎯 Proof Outcome:
  ❌ STUCK: 5 branch(es) got stuck

```

# Proof test_process_thaw_account                                   

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   691
├─ 4  🔀 SPLIT     ((split))            │ line  5710
┃  ├─ 5     NORMAL                        │ line 10741
┃  ├─ 7     NORMAL                        │ line 15769
┃  ├─ 9     NORMAL                        │ line 20799
┃  ├─ 10 🔀 SPLIT     ((split))            │ line 26115
┃  ┃  ├─ 11    NORMAL                        │ line 31137
┃  ┃  ├─ 13 🔀 SPLIT     ((split))            │ line 36156
┃  ┃  ┃  ├─ 15    NORMAL                        │ line 41186
┃  ┃  ┃  ├─ 19    NORMAL                        │ line 46213
┃  ┃  ┃  └─ 23 ❌ STUCK     ((stuck, leaf))      │ line 51235
┃  ┃     ├─ 16    NORMAL                        │ line 56272
┃  ┃     └─ 20 ❌ STUCK     ((stuck, leaf))      │ line 61299
┃     ├─ 12    NORMAL                        │ line 66337
┃     ├─ 14 🔀 SPLIT     ((split))            │ line 71356
┃     ┃  ├─ 17    NORMAL                        │ line 76386
┃     ┃  ├─ 21    NORMAL                        │ line 81413
┃     ┃  └─ 24 ❌ STUCK     ((stuck, leaf))      │ line 86435
┃        ├─ 18    NORMAL                        │ line 91472
┃        └─ 22 ❌ STUCK     ((stuck, leaf))      │ line 96499
   ├─ 6     NORMAL                        │ line 101537
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 106565
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 111623

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 24

  ⚪ Normal      :  13
  🌳 Root        :   2
  🔀 Split       :   4
  ❌ Stuck       :   5

🎯 Proof Outcome:
  ❌ STUCK: 5 branch(es) got stuck

```

# Proof test_process_transfer_checked                               

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   941
├─ 4     NORMAL                        │ line  7360
├─ 5  🔀 SPLIT     ((split))            │ line 13084
┃  ├─ 6     NORMAL                        │ line 18809
┃  ├─ 8     NORMAL                        │ line 24531
┃  ├─ 10 🔀 SPLIT     ((split))            │ line 30256
┃  ┃  ├─ 11    NORMAL                        │ line 35983
┃  ┃  ├─ 13    NORMAL                        │ line 41707
┃  ┃  ├─ 15    NORMAL                        │ line 48156
┃  ┃  ├─ 16    NORMAL                        │ line 54616
┃  ┃  ├─ 17 🔀 SPLIT     ((split))            │ line 60348
┃  ┃  ┃  ├─ 18    NORMAL                        │ line 66089
┃  ┃  ┃  ├─ 20    NORMAL                        │ line 71827
┃  ┃  ┃  ┃  ├─ 22    NORMAL                        │ line 77692
┃  ┃  ┃  ┃  └─ 26 ⏳ PENDING   ((leaf, pending))    │ line 83556
┃  ┃  ┃  ┃  └─ 23 ⏳ PENDING   ((leaf, pending))    │ line 89298
┃  ┃  ┃     └─ 24 ⏳ PENDING   ((leaf, pending))    │ line 95164
┃  ┃     ├─ 19    NORMAL                        │ line 101034
┃  ┃     ├─ 21    NORMAL                        │ line 106772
┃  ┃     └─ 25 ⏳ PENDING   ((leaf, pending))    │ line 113238
┃     ├─ 12    NORMAL                        │ line 118995
┃     └─ 14 ❌ STUCK     ((stuck, leaf))      │ line 124719
   ├─ 7     NORMAL                        │ line 130476
   └─ 9  ❌ STUCK     ((stuck, leaf))      │ line 136198
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 141950

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 26

  ⚪ Normal      :  15
  ⏳ Pending     :   4
  🌳 Root        :   2
  🔀 Split       :   3
  ❌ Stuck       :   2

🎯 Proof Outcome:
  ❌ STUCK: 2 branch(es) got stuck
  ⏳ PENDING: 4 branch(es) still pending

```

# Proof test_process_transfer                                       

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   719
├─ 4  🔀 SPLIT     ((split))            │ line  6820
┃  ├─ 5     NORMAL                        │ line 12267
┃  ├─ 7     NORMAL                        │ line 17711
┃  ├─ 9  🔀 SPLIT     ((split))            │ line 23158
┃  ┃  ├─ 10    NORMAL                        │ line 28607
┃  ┃  ├─ 12    NORMAL                        │ line 34053
┃  ┃  ├─ 14    NORMAL                        │ line 40171
┃  ┃  ├─ 15    NORMAL                        │ line 46300
┃  ┃  ├─ 16 🔀 SPLIT     ((split))            │ line 51754
┃  ┃  ┃  ├─ 17    NORMAL                        │ line 57217
┃  ┃  ┃  ├─ 19    NORMAL                        │ line 62677
┃  ┃  ┃  ┃  ├─ 21    NORMAL                        │ line 68264
┃  ┃  ┃  ┃  └─ 25 ⏳ PENDING   ((leaf, pending))    │ line 73850
┃  ┃  ┃  ┃  ├─ 22    NORMAL                        │ line 79314
┃  ┃  ┃  ┃  ┃  └─ 26 ⏳ PENDING   ((leaf, pending))    │ line 84903
┃  ┃  ┃  ┃  ┃  └─ 27 ⏳ PENDING   ((leaf, pending))    │ line 90492
┃  ┃  ┃  ┃     └─ 28 ⏳ PENDING   ((leaf, pending))    │ line 96082
┃  ┃  ┃     └─ 23 ⏳ PENDING   ((leaf, pending))    │ line 101674
┃  ┃     ├─ 18    NORMAL                        │ line 107266
┃  ┃     ├─ 20    NORMAL                        │ line 112726
┃  ┃     └─ 24 ⏳ PENDING   ((leaf, pending))    │ line 118861
┃     ├─ 11    NORMAL                        │ line 124340
┃     └─ 13 ❌ STUCK     ((stuck, leaf))      │ line 129786
   ├─ 6     NORMAL                        │ line 135265
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 140709
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 146183

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 28

  ⚪ Normal      :  15
  ⏳ Pending     :   6
  🌳 Root        :   2
  🔀 Split       :   3
  ❌ Stuck       :   2

🎯 Proof Outcome:
  ❌ STUCK: 2 branch(es) got stuck
  ⏳ PENDING: 6 branch(es) still pending

```

# Proof test_process_ui_amount_to_amount                            

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
└─ 3  ❌ STUCK     ((stuck, leaf))      │ line   258
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line  4943

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 3

  🌳 Root        :   2
  ❌ Stuck       :   1

🎯 Proof Outcome:
  ❌ STUCK: 1 branch(es) got stuck

```

# Proof test_process_withdraw_excess_lamports                       

```
🌲 PROOF TREE STRUCTURE
════════════════════════════════════════════════════════════════════════════════

Format: Node ID | Type | Attributes | Line Number
────────────────────────────────────────────────────────────────────────────────

┌─ 1  🌳 ROOT      ((root, init))       │ line     2
├─ 3     NORMAL                        │ line   691
├─ 4  🔀 SPLIT     ((split))            │ line  6077
┃  ├─ 5     NORMAL                        │ line 11475
┃  ├─ 7     NORMAL                        │ line 16870
┃  ├─ 9     NORMAL                        │ line 22914
┃  └─ 10 ❌ STUCK     ((stuck, leaf))      │ line 28314
   ├─ 6     NORMAL                        │ line 33777
   └─ 8  ❌ STUCK     ((stuck, leaf))      │ line 39172
┌─ 2  🌳 ROOT      ((root, leaf, target, terminal)) │ line 44597

📊 STATISTICS
════════════════════════════════════════
📈 Total nodes: 10

  ⚪ Normal      :   5
  🌳 Root        :   2
  🔀 Split       :   1
  ❌ Stuck       :   2

🎯 Proof Outcome:
  ❌ STUCK: 2 branch(es) got stuck

```

