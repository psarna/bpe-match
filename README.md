# BPE matcher for pretokenization

Replacement for the notorious `const GPT4_PATTERN: &str = r"'(?i:[sdmt]|ll|ve|re)|[^\r\n\p{L}\p{N}]?+\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]++[\r\n]*|\s*[\r\n]|\s+(?!\S)|\s+";`

When https://github.com/karpathy/nanochat uses it instead of regex, I get the following improvement:

Old:
```
📊 Performance comparison:
   RustBPE: 0.5127s
   HuggingFace: 2.2548s
   Speedup: 4.40x
```

New and fancy:
```
📊 Performance comparison:
   RustBPE: 2.7347s
   HuggingFace: 23.9614s
   Speedup: 8.76x
```
