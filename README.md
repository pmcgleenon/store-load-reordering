# store-load-reordering
A simple example to demonstrate store-load memory re-ordering on x86/x64 processors using Rust.  
This is an example of x86 CPU instruction reordering and highlights the behaviour of the store buffer.

Inspired by [Jeff Preshing's blog Memory Reordering Caught in the Act](https://preshing.com/20120515/memory-reordering-caught-in-the-act/)
let's recreate the infamous X86 Store/Load Reordering in Rust and explore what's happening 


# Building the code
You can build the code and generate Assembly.  The assembly output will be available at `target/release/deps/store_load_reordering-<hash>.s`
```
cargo rustc --release -- --emit asm
```

# Running the code

Use the -h option to display the command line options.  The available options for ordering are:
  - Relaxed (default)
  - AcquireRelease (use Acquire ordering on Load and Release ordering on Store)
  - SeqCst 

```
store-load-reordering -h
A tool to demonstrate memory ordering effects.

Usage: store-load-reordering [OPTIONS]

Options:
  -o, --ordering <ORDERING>  Memory Ordering to use [default: Relaxed]
  -b, --barrier
  -h, --help                 Print help
  -V, --version              Print version
```

Run the code with the default relaxed ordering
```
store-load-reordering
```
Run the code with Sequentially Consistent ordering
```
store-load-reordering -o SeqCst
```
Run the code with AcquireRelease ordering
```
store-load-reordering -o AcquireRelease
```
Run the code with Relaxed ordering and a memory barrier
```
store-load-reordering -b
```

# Assembly output
X86 instructions for both relaxed and acquire-release ordering are the same.   The SeqCst x86 instructions use the `xchg` instruction which has an implicit lock.
The `mfence` instruction is used to prevent reordering - only with a SeqCst fence.

ARM on the other hand uses the same instructions for acquire-release ordering and sequential consistency.  

| Rust Code                                                                                                                                                                                                                            | X86                                                                                                                        | Arm                                                                                                 |
|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|:---------------------------------------------------------------------------------------------------------------------------|:----------------------------------------------------------------------------------------------------|
| <pre>pub fn relaxed(x: &AtomicU32, y: &AtomicU32, r1: &AtomicU32 ) {<br>        x.store(1, Ordering::Relaxed);<br>        r1.store(y.load(Ordering::Relaxed), Ordering::Relaxed);<br>}</pre>                                         | <pre>mov     DWORD PTR [rdi], 1<br>mov     eax, DWORD PTR [rsi]<br>mov     DWORD PTR [rdx], eax</pre>                      | <pre>mov     w0, 1<br>str     w0, [x0]<br>ldr     w0, [x1]<br>str     w0, [x2]</pre>                |
| <pre>pub fn acquire_release(x: &AtomicU32, y: &AtomicU32, r1: &AtomicU32 ) {<br>        x.store(1, Ordering::Release);<br>        r1.store(y.load(Ordering::Acquire), Ordering::Release);<br>}</pre>                                 | <pre>mov     DWORD PTR [rdi], 1<br>mov     eax, DWORD PTR [rsi]<br>mov     DWORD PTR [rdx], eax</pre>                      | <pre>mov     w0, 1<br>stlr    w0, [x0]<br>ldar    w0, [x1]<br>stlr    w0, [x2]</pre>                |
| <pre>pub fn sequential_consistent(x: &AtomicU32, y: &AtomicU32, r1: &AtomicU32 ) {<br>        x.store(1, Ordering::SeqCst);<br>        r1.store(y.load(Ordering::SeqCst), Ordering::SeqCst);<br>}</pre>                              | <pre>mov     eax, 1<br>xchg    dword ptr [rdi], eax<br>mov     eax, dword ptr [rsi]<br>xchg    dword ptr [rdx], eax</pre>  | <pre>mov     w0, 1<br>stlr    w0, [x0]<br>ldar    w0, [x1]<br>stlr    w0, [x2]</pre>                | 
| <pre>pub fn relaxed_with_barrier_seqcst(x: &AtomicU32, y: &AtomicU32, r1: &AtomicU32 ) {<br>        x.store(1, Ordering::Relaxed);<br>        fence(Ordering::SeqCst);<br>        r1.store(y.load(Ordering::Relaxed), Ordering::Relaxed);<br>}</pre> | <pre>mov     DWORD PTR [rdi], 1<br>mfence<br>mov     eax, DWORD PTR [rsi]<br>mov     DWORD PTR [rdx], eax</pre>            | <pre>mov     w0, 1<br>str     w0, [x0]<br>dsb     ish<br>ldr     w0, [x1]<br>str     w0, [x2]</pre> |
| <pre>pub fn relaxed_with_barrier(x: &AtomicU32, y: &AtomicU32, r1: &AtomicU32 ) {<br>        x.store(1, Ordering::Relaxed);<br>        fence(Ordering::Release);<br>        r1.store(y.load(Ordering::Relaxed), Ordering::Relaxed);<br>}</pre> | <pre>mov     DWORD PTR [rdi], 1<br>mov     eax, DWORD PTR [rsi]<br>mov     DWORD PTR [rdx], eax</pre>           | <pre>mov     w0, 1<br>str     w0, [x0]<br>dsb     ish<br>ldr     w0, [x1]<br>str     w0, [x2]</pre> |


# Godbolt X86 link 
https://rust.godbolt.org/z/WKEeTMWGz

# Godbolt Arm link
https://rust.godbolt.org/z/sqo6KcW1K


# Useful Links - Background Info
[Examining ARM vs X86 Memory Models with Rust](https://www.nickwilcox.com/blog/arm_vs_x86_memory_model/)

[Rust atomics on x86: How and why](https://darkcoding.net/software/rust-atomics-on-x86/)

[Explaining Atomics in Rust](https://cfsamsonbooks.gitbook.io/explaining-atomics-in-rust)

[Rust release and acquire memory ordering by example](https://medium.com/@omid.jn/rust-release-and-acquire-memory-ordering-by-example-d8de58ef4e36)

[Rust Atomics and Locks](https://marabos.nl/atomics/)
