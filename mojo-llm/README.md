# mojo-llm

A CPU-first native Mojo experiment for small language models. For now it builds
an empty executable: no weights, GGUF loader, GPU runtime, or inference.
“Static” means future kernels can specialize dimensions and bound their buffers
at compile time, not that weights are embedded or the executable is fully
statically linked. llama.cpp/ggml is a reference, not a dependency.

From this directory on the Linux desktop, with Pixi and a system C linker:

```sh
pixi run --locked smoke
```

This builds `build/mojo-llm` and runs it. Expect `mojo-llm: scaffold OK` and
exit status zero. Use `pixi run --locked build` to build without running.

For a native Jetson Orin test, copy the sources, manifest and lockfile (not
`.pixi/` or the desktop executable), then run the same smoke command on the
board. The ARM64 lockfile is not an on-device compatibility test.

Next experiment: decode one synthetic Q4_K block and compare it with a pinned
ggml reference. This tests one operation toward Qwen3.5-2B Q4_K_M, not a full
model loader or generator.
