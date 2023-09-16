# An introduction to ipnsort

Author: Lukas Bergdoll @Voultapher  
Date: TODO (DD-MM-YYYY)

This is an introduction and overview of a efficient, generic and robust unstable sort implementation called ipnsort (instruction-parallel-network sort) by Lukas Bergdoll [source code](https://github.com/Voultapher/sort-research-rs/tree/main/ipnsort).

TL;DR: ipnsort improves upon `slice::sort_unstable` in variety a of ways.

---

Bias disclaimer. The author of this writeup is the author of ipnsort.

## Design goals

The primary goal was to develop a replacement for the current Rust standard library `slice::sort_unstable`.

- **Correct**: Correct ordered output if the user-defined comparison function implements a total order.
- **Safe**: panic- observation- and Ord-violation-safe for any input. Zero UB regardless of input and user-defined comparison function. TODO link safety writeup.
- **Deterministic**: Same output every time, on every machine and ISA.
- **Hardware agnostic**: No architecture specific code.
- **Efficient**: Race to sleep and focus on instruction-level-parallelism (ILP) over SIMD.
- **Generic**: Works the same way for builtin types and user-defined types. Supports arbitrary comparison functions.
- **Robust**:
  - Optimized and tested along these dimensions:
    - Input length (0-1e7)
    - Input type (integer like, medium sized, large sized)
    - Input pattern (fully random, Zipfian distributions, low cardinality, presorted + append, and more)
    - CPU prediction state (hot loop only doing sort, cold code with i-cache and d-cache misses)
  - Guaranteed O(N * log(N)) worst case comparisons.
  - Guaranteed O(N) comparisons for fully ascending and descending inputs.
  - No performance cliffs.
- **Binary-size**: Relatively small binary-size for types like `u64` and `String`. i-cache is a shared resource and the program will likely do more than just sort.
- **Compile-time**: At least as fast to compile as the current `slice::sort_unstable` if hardware-parallelism is available and not much worse if not. Both debug and release configurations.
- **In-place**: No heap allocations. And explicit stack usage should be limited to a couple kB.
- **Debug performance**: Performance of un-optimized binaries should not be much worse than the current `slice::sort_unstable`.

## Design non-goals

- **Fastest non-generic integer sort**: The staked design goals in combination are incompatible with this goal. The author is aware of simple ways to improve integer performance by 10-20% if some of the design goals are ignored. For best in-class performance [vqsort](https://github.com/google/highway/tree/master/hwy/contrib/sort) by Mark Blacher, Joachim Giesen, Peter Sanders, Jan Wassenberg assuming AVX2+ and Clang. Especially for small types `mem::size_of::<T>() < mem::size_of::<u64>()` a good radix sort is likely faster. More info here [10~17x faster than what? A performance analysis of Intel's x86-simd-sort (AVX-512)](https://github.com/Voultapher/sort-research-rs/blob/main/writeup/intel_avx512/text.md).
- **Tiny binary-size**: Implementation complexity and binary-size are related, for projects that care about binary-size and or compile-time above everything else [tiny_sort](https://github.com/Voultapher/tiny-sort-rs) is a better fit.
- **Varied compilers**: Only rustc using LLVM was tested and designed for.


## High level overview

The starting point for this journey was the source code of `slice::sort_unstable`. Which itself is mostly a port of [pdqsort](https://github.com/orlp/pdqsort) by Orson Peters.

Modern high-performance sort implementations combine various strategies to exploit input patterns and hardware capabilities. In effect this makes all of them hybrid algorithms. For this reason it is more appropriate to talk about sort implementations and their components instead of a "sort algorithm".


### pdqsort

To better understand ipnsort, we start with a component level overview of pdqsort

- **Partial insertion sort**: Up to 5 out-of-order elements are handled by insertion sort if everything else is already sorted.
- **Quicksort**: Top level loop and algorithm.
- **Insertion sort**: Recursion stopper small-sort.
- **Ancestor pivot tracking**: Common element filtering allowing for O(K * log(N)) total comparisons where K is the number of distinct elements. Also great for Zipfian distributions.
- **Partition**: Hoare partition with block offset computation and cyclic permutation element swapping derived from [BlockQuicksort](https://arxiv.org/pdf/1604.06697.pdf) by Stefan Edelkamp, Armin Weiß.
- **Equal partition**: Branchy Hoare partition for filtering out common elements.
- **Pattern breaker**: Xorshift RNG element shuffling to avoid fallback.
- **Heapsort**: Branchy Heapsort fallback triggered by excess poor partitions.

### ipnsort

ipnsort performs type introspection to specialize for different kinds of types, based on heuristics looking at `mem::size_of::<T>()` and the traits `Copy` and `Freeze`. A component that is a good fit for an integer may not be a good fit for a heap backed string, or a large stack array.

- **Small total len 

## Verification

### Correct

### Safe

TODO

## Author's conclusion and opinion

A sort implementation is a bit like a battery. It's relatively easy to be good in one specific property, but being good in most aspects and bad in none is *a lot* harder. The work on ipnsort consumed more than a 1000 hours of my personal time in 2022 and 2023, I'm happy to have achieved my ambitious goals and even managed to innovate some aspects.

## Thanks
