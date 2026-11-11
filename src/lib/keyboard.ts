// MCHOSE K99 V2 键盘布局与键码。
//
// 键名表（113 槽）与物理布局来自公开的 MCHOSE 网页驱动快照
// （wzk112/MchoseHub 的键盘资源；布局算法见其 KeyboardLayout.swift）。
// 这里的代码是重新实现的展示/编辑逻辑。

export const KEY_NAMES: string[] = [
"Esc",
"backquote",
"Tab",
"Caps",
"Left_Shift",
"Left_Ctrl",
"F1",
"one",
"Q",
"A",
"Z",
"Left_Win",
"F2",
"two",
"W",
"S",
"X",
"Left_Alt",
"F3",
"three",
"E",
"D",
"C",
"space1",
"F4",
"four",
"R",
"F",
"V",
"space2",
"F5",
"five",
"T",
"G",
"B",
"Space",
"F6",
"six",
"Y",
"H",
"N",
"space3",
"F7",
"seven",
"U",
"J",
"M",
"space4",
"F8",
"eight",
"I",
"K",
"comma",
"Right_Alt",
"F9",
"nine",
"O",
"L",
"dot",
"Fn",
"F10",
"zero",
"P",
"semicolon",
"slash",
"Right_Ctrl",
"F11",
"transverse",
"left_bracket",
"singlequotes",
"space5",
"space6",
"F12",
"equal",
"right_bracket",
"space7",
"space8",
"space9",
"Delete",
"Backspace",
"backslash",
"Enter",
"Right_Shift",
"left",
"space10",
"space11",
"space12",
"space13",
"up",
"down",
"Home",
"Num",
"num_seven",
"num_four",
"num_one",
"right",
"End",
"num_divide",
"num_eight",
"num_five",
"num_two",
"num_zero",
"PgUp",
"num_multiply",
"num_nine",
"num_six",
"num_three",
"num_dot",
"PgDn",
"num_subtract",
"num_add",
"space14",
"num_Enter"
];

export const BOARD = [
{
"key": "Esc",
"label": "Esc",
"left": 0,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
41
],
"fn": [
7,
0,
0,
4
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F1",
"label": "F1",
"left": 13,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
58
],
"fn": [
2,
0,
0,
112
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F2",
"label": "F2",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
59
],
"fn": [
2,
0,
0,
111
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F3",
"label": "F3",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
60
],
"fn": [
0,
8,
0,
43
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F4",
"label": "F4",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
61
],
"fn": [
0,
8,
0,
8
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F5",
"label": "F5",
"left": 13,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
62
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F6",
"label": "F6",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
63
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F7",
"label": "F7",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
64
],
"fn": [
2,
0,
0,
182
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F8",
"label": "F8",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
65
],
"fn": [
2,
0,
0,
205
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F9",
"label": "F9",
"left": 13,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
66
],
"fn": [
2,
0,
0,
181
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F10",
"label": "F10",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
67
],
"fn": [
2,
0,
0,
226
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F11",
"label": "F11",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
68
],
"fn": [
2,
0,
0,
234
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F12",
"label": "F12",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
69
],
"fn": [
2,
0,
0,
233
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Delete",
"label": "Delete",
"left": 13,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
76
],
"fn": [
8,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Home",
"label": "Home",
"left": 24,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
74
],
"fn": [
8,
0,
0,
2
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "End",
"label": "End",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
77
],
"fn": [
8,
0,
0,
1
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "PgUp",
"label": "PgUp",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
75
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "PgDn",
"label": "PgDn",
"left": 4,
"top": 0,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
78
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "backquote",
"label": "` ～",
"left": 0,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
53
],
"fn": [
7,
0,
0,
8
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "one",
"label": "1 !",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
30
],
"fn": [
7,
0,
0,
5
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "two",
"label": "2 @",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
31
],
"fn": [
7,
0,
0,
6
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "three",
"label": "3 #",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
32
],
"fn": [
7,
0,
0,
7
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "four",
"label": "4 $",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
33
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "five",
"label": "5 %",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
34
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "six",
"label": "6 ^",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
35
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "seven",
"label": "7 &",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
36
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "eight",
"label": "8 *",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
37
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "nine",
"label": "9 (",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
38
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "zero",
"label": "0 )",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
39
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "transverse",
"label": "- _",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
45
],
"fn": [
8,
2,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "equal",
"label": "= +",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
46
],
"fn": [
8,
2,
0,
2
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Backspace",
"label": "Backspace",
"left": 4,
"top": 18,
"width": 78,
"height": 42,
"def": [
0,
0,
0,
42
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Num",
"label": "Num",
"left": 24,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
83
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_divide",
"label": "/",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
84
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_multiply",
"label": "*",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
85
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_subtract",
"label": "-",
"left": 4,
"top": 18,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
86
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Tab",
"label": "Tab",
"left": 0,
"top": 4,
"width": 64,
"height": 42,
"def": [
0,
0,
0,
43
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Q",
"label": "Q",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
20
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "W",
"label": "W",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
26
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "E",
"label": "E",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
8
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "R",
"label": "R",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
21
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "T",
"label": "T",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
23
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Y",
"label": "Y",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
28
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "U",
"label": "U",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
24
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "I",
"label": "I",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
12
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "O",
"label": "O",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
18
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "P",
"label": "P",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
19
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "left_bracket",
"label": "[ {",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
47
],
"fn": [
8,
3,
2,
1
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "right_bracket",
"label": "] }",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
48
],
"fn": [
8,
3,
1,
1
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "backslash",
"label": "\\ |",
"left": 4,
"top": 4,
"width": 56,
"height": 42,
"def": [
0,
0,
0,
49
],
"fn": [
8,
2,
0,
1
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_seven",
"label": "7",
"left": 24,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
95
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_eight",
"label": "8",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
96
],
"fn": [
8,
3,
1,
2
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_nine",
"label": "9",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
97
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_add",
"label": "+",
"left": 4,
"top": 4,
"width": 42,
"height": 88,
"def": [
0,
0,
0,
87
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Caps",
"label": "Caps",
"left": 0,
"top": -42,
"width": 76,
"height": 42,
"def": [
0,
0,
0,
57
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "A",
"label": "A",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
4
],
"fn": [
7,
0,
0,
24
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "S",
"label": "S",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
22
],
"fn": [
7,
0,
0,
26
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "D",
"label": "D",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
7
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "F",
"label": "F",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
9
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "G",
"label": "G",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
10
],
"fn": [
7,
0,
0,
10
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "H",
"label": "H",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
11
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "J",
"label": "J",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
13
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "K",
"label": "K",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
14
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "L",
"label": "L",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
15
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "semicolon",
"label": "; :",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
51
],
"fn": [
8,
4,
2,
1
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "singlequotes",
"label": "‘ ”",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
52
],
"fn": [
8,
4,
1,
1
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Enter",
"label": "Enter",
"left": 4,
"top": -42,
"width": 90,
"height": 42,
"def": [
0,
0,
0,
40
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_four",
"label": "4",
"left": 24,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
92
],
"fn": [
8,
4,
2,
2
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_five",
"label": "5",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
93
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_six",
"label": "6",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
94
],
"fn": [
8,
4,
1,
2
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Left_Shift",
"label": "Shift",
"left": 0,
"top": 4,
"width": 97,
"height": 42,
"def": [
0,
2,
0,
0
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Z",
"label": "Z",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
29
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "X",
"label": "X",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
27
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "C",
"label": "C",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
6
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "V",
"label": "V",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
25
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "B",
"label": "B",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
5
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "N",
"label": "N",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
17
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "M",
"label": "M",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
16
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "comma",
"label": ", <",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
54
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "dot",
"label": ". >",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
55
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "slash",
"label": "/ ?",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
56
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Right_Shift",
"label": "Shift",
"left": 4,
"top": 4,
"width": 71,
"height": 42,
"def": [
0,
32,
0,
0
],
"fn": [
7,
0,
0,
11
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "up",
"label": "↑",
"left": 13,
"top": 13,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
82
],
"fn": [
8,
3,
1,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_one",
"label": "1",
"left": 13,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
89
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_two",
"label": "2",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
90
],
"fn": [
8,
3,
2,
2
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_three",
"label": "3",
"left": 4,
"top": 4,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
91
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_Enter",
"label": "Enter",
"left": 4,
"top": 4,
"width": 42,
"height": 88,
"def": [
0,
0,
0,
88
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Left_Ctrl",
"label": "Ctrl",
"left": 0,
"top": -42,
"width": 54,
"height": 42,
"def": [
0,
1,
0,
0
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Left_Win",
"label": "Win",
"left": 4,
"top": -42,
"width": 54,
"height": 42,
"def": [
0,
8,
0,
0
],
"fn": [
7,
0,
0,
1
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Left_Alt",
"label": "Alt",
"left": 4,
"top": -42,
"width": 54,
"height": 42,
"def": [
0,
4,
0,
0
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Space",
"label": "Space",
"left": 4,
"top": -42,
"width": 274,
"height": 42,
"def": [
0,
0,
0,
44
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Right_Alt",
"label": "Alt",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
64,
0,
0
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Fn",
"label": "Fn",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
13,
0,
0,
0
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "Right_Ctrl",
"label": "Ctrl",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
16,
0,
0
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "left",
"label": "←",
"left": 13,
"top": -33,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
80
],
"fn": [
8,
4,
2,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "down",
"label": "↓",
"left": 4,
"top": -33,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
81
],
"fn": [
8,
3,
2,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "right",
"label": "→",
"left": 4,
"top": -33,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
79
],
"fn": [
8,
4,
1,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_zero",
"label": "0",
"left": 13,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
98
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
},
{
"key": "num_dot",
"label": ".",
"left": 4,
"top": -42,
"width": 42,
"height": 42,
"def": [
0,
0,
0,
99
],
"fn": [
0,
0,
0,
0
],
"fn2": [
0,
0,
0,
0
]
}
] as const;

export type BoardKey = (typeof BOARD)[number];
export type KeyCode = [number, number, number, number];

/** 按下标查协议槽位；-1 表示这枚键在 113 槽里没有对应项。 */
export function keyIndex(key: string): number {
  return KEY_NAMES.indexOf(key);
}

export type Placement = {
  key: string;
  label: string;
  index: number;
  x: number;
  y: number;
  w: number;
  h: number;
  def: KeyCode;
  fn: KeyCode;
  fn2: KeyCode;
};

const ROW_STARTS = new Set(['backquote', 'Tab', 'Caps', 'Left_Shift', 'Left_Ctrl']);
const UNIT = 38;
const GAP = 4;

/** 复刻官方布局的累加规则；单位是 px（最后再归一化成百分比）。 */
export function layoutKeys(): { items: Placement[]; width: number; height: number } {
  let row = 0;
  let cursor = 0;
  let width = 0;
  let height = 0;
  const items: Placement[] = [];
  for (const k of BOARD) {
    if (ROW_STARTS.has(k.key)) {
      row += 1;
      cursor = 0;
    }
    cursor += k.left / 46;
    const slotWidth = (k.width + 4) / 46;
    const x = cursor * UNIT;
    const y = row * UNIT;
    const w = slotWidth * UNIT - GAP;
    const h = (k.height / 42) * UNIT - GAP;
    items.push({
      key: k.key,
      label: k.label,
      index: keyIndex(k.key),
      x,
      y,
      w,
      h,
      def: [...k.def] as KeyCode,
      fn: [...k.fn] as KeyCode,
      fn2: [...k.fn2] as KeyCode,
    });
    width = Math.max(width, (cursor + slotWidth) * UNIT);
    height = Math.max(height, y + h);
    cursor += slotWidth;
  }
  return { items, width, height };
}

export const codeHex = (c: KeyCode) =>
  c.map((b) => b.toString(16).padStart(2, '0')).join('');

export function parseCode(hex: string): KeyCode | null {
  const clean = hex.trim().toLowerCase().replace(/^0x/, '').replace(/[^0-9a-f]/g, '');
  if (clean.length !== 8) return null;
  const out: number[] = [];
  for (let i = 0; i < 8; i += 2) out.push(parseInt(clean.slice(i, i + 2), 16));
  return out as KeyCode;
}

const HID: Record<number, string> = {
  0x04: 'A', 0x05: 'B', 0x06: 'C', 0x07: 'D', 0x08: 'E', 0x09: 'F',
  0x0a: 'G', 0x0b: 'H', 0x0c: 'I', 0x0d: 'J', 0x0e: 'K', 0x0f: 'L',
  0x10: 'M', 0x11: 'N', 0x12: 'O', 0x13: 'P', 0x14: 'Q', 0x15: 'R',
  0x16: 'S', 0x17: 'T', 0x18: 'U', 0x19: 'V', 0x1a: 'W', 0x1b: 'X',
  0x1c: 'Y', 0x1d: 'Z',
  0x1e: '1', 0x1f: '2', 0x20: '3', 0x21: '4', 0x22: '5', 0x23: '6',
  0x24: '7', 0x25: '8', 0x26: '9', 0x27: '0',
  0x28: 'Enter', 0x29: 'Esc', 0x2a: 'Backspace', 0x2b: 'Tab', 0x2c: 'Space',
  0x2d: '-', 0x2e: '=', 0x2f: '[', 0x30: ']', 0x31: '\\', 0x33: ';',
  0x34: "'", 0x35: '`', 0x36: ',', 0x37: '.', 0x38: '/', 0x39: 'Caps',
  0x3a: 'F1', 0x3b: 'F2', 0x3c: 'F3', 0x3d: 'F4', 0x3e: 'F5', 0x3f: 'F6',
  0x40: 'F7', 0x41: 'F8', 0x42: 'F9', 0x43: 'F10', 0x44: 'F11', 0x45: 'F12',
  0x49: 'Insert', 0x4a: 'Home', 0x4b: 'PgUp', 0x4c: 'Delete', 0x4d: 'End',
  0x4e: 'PgDn', 0x4f: '→', 0x50: '←', 0x51: '↓', 0x52: '↑',
  0x53: 'Num', 0x54: 'Num /', 0x55: 'Num *', 0x56: 'Num -', 0x57: 'Num +',
  0x58: 'Num Enter', 0x59: 'Num 1', 0x5a: 'Num 2', 0x5b: 'Num 3',
  0x5c: 'Num 4', 0x5d: 'Num 5', 0x5e: 'Num 6', 0x5f: 'Num 7',
  0x60: 'Num 8', 0x61: 'Num 9', 0x62: 'Num 0', 0x63: 'Num .',
  0xcd: '播放/暂停', 0xb5: '下一曲', 0xb6: '上一曲', 0xb7: '停止',
  0xe2: '静音', 0xe9: '音量 +', 0xea: '音量 -',
};

const MODS: Record<number, string> = {
  0x01: '左 Ctrl', 0x02: '左 Shift', 0x04: '左 Alt', 0x08: '左 Win',
  0x10: '右 Ctrl', 0x20: '右 Shift', 0x40: '右 Alt', 0x80: '右 Win',
};

export function codeLabel(code: KeyCode): string {
  if (code.every((b) => b === 0)) return '默认';
  if (code[0] === 0x0d) return 'Fn';
  if (code[0] === 0) {
    if (code[1] !== 0 && code[2] === 0) return MODS[code[1]] ?? `修饰键 0x${code[1].toString(16)}`;
    if (code[2] === 0 && code[3] !== 0) return HID[code[3]] ?? `HID 0x${code[3].toString(16)}`;
  }
  if (code[0] === 2 && code[3] !== 0) return HID[code[3]] ?? `多媒体 0x${code[3].toString(16)}`;
  return codeHex(code);
}

export type ActionOption = { label: string; code: KeyCode };

const key = (label: string, usage: number): ActionOption => ({
  label,
  code: [0, 0, 0, usage],
});

export const ACTIONS: ActionOption[] = [
  { label: '默认（不修改）', code: [0, 0, 0, 0] },
  ...'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').map((c) => key(c, 0x04 + c.charCodeAt(0) - 65)),
  ...'1234567890'.split('').map((c, i) => key(c, 0x1e + i)),
  ...Array.from({ length: 12 }, (_, i) => key(`F${i + 1}`, 0x3a + i)),
  key('Esc', 0x29), key('Tab', 0x2b), key('Caps', 0x39), key('Enter', 0x28),
  key('Space', 0x2c), key('Backspace', 0x2a), key('Delete', 0x4c),
  key('Insert', 0x49), key('Home', 0x4a), key('End', 0x4d),
  key('PgUp', 0x4b), key('PgDn', 0x4e),
  key('←', 0x50), key('→', 0x4f), key('↑', 0x52), key('↓', 0x51),
  { label: '左 Ctrl', code: [0, 1, 0, 0] },
  { label: '左 Shift', code: [0, 2, 0, 0] },
  { label: '左 Alt', code: [0, 4, 0, 0] },
  { label: '左 Win', code: [0, 8, 0, 0] },
  { label: '右 Ctrl', code: [0, 16, 0, 0] },
  { label: '右 Shift', code: [0, 32, 0, 0] },
  { label: '右 Alt', code: [0, 64, 0, 0] },
  { label: 'Fn', code: [0x0d, 0, 0, 0] },
  { label: '音量 +', code: [2, 0, 0, 0xe9] },
  { label: '音量 -', code: [2, 0, 0, 0xea] },
  { label: '静音', code: [2, 0, 0, 0xe2] },
  { label: '播放/暂停', code: [2, 0, 0, 0xcd] },
  { label: '上一曲', code: [2, 0, 0, 0xb6] },
  { label: '下一曲', code: [2, 0, 0, 0xb5] },
];

const LAYOUT = layoutKeys();

export function defaultLayer(layer: number): KeyCode[] {
  const out: KeyCode[] = Array.from({ length: KEY_NAMES.length }, () => [0, 0, 0, 0]);
  for (const p of LAYOUT.items) {
    if (p.index < 0) continue;
    const board = BOARD.find((b) => b.key === p.key);
    if (!board) continue;
    out[p.index] =
      layer === 0
        ? ([...board.def] as KeyCode)
        : layer === 1
          ? ([...board.fn] as KeyCode)
          : ([...board.fn2] as KeyCode);
  }
  return out;
}

export const BOARD_SIZE = { width: LAYOUT.width, height: LAYOUT.height };

const SHORT_LABELS: Record<string, string> = {
  Backspace: '⌫',
  Delete: 'Del',
  Space: 'Space',
  '` ～': '`',
};

/** 键帽上只放得下很短的名字。 */
export function keyLabel(label: string): string {
  if (SHORT_LABELS[label]) return SHORT_LABELS[label];
  const first = label.split(' ')[0];
  return first.length > 5 ? first.slice(0, 5) : first;
}
