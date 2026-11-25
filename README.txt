XCL 2D texture UMax backend comparison

The same synthetic RDNA2 program was translated through the default HLSL path and the direct DXIL path on three source revisions. Each program consists of IMAGE_ATOMIC_UMAX and S_ENDPGM. The operands are fixed and only DMASK and GLC vary. The probe compiles HLSL to cs_6_6 with DXC, calls the DXIL validator and disassembler on each result, and finds the atomic unsigned-maximum intrinsic in that module. Three different programs per revision produce six validated shader modules.

The 32-bit control (DMASK 1, GLC 0) emits i32 for both backends on all revisions. The two wide cases (DMASK 3, GLC 0 or 1) require i64. The table shows their actual width. The printed "match" compares width only. All modules passed DXIL validation, including the wrong-width modules.

Source                              HLSL through DXC    Direct DXIL
XCL main 80f48190                   i32                 i32
XCL PR #1948 1e4f2bda                i32                 i64
XCL PR #1541 a4f78345                i64                 i32

The observed/ directory contains the three concise run logs and their generated shader disassembly files. probe.rs and the locked dependencies in locks/ are sufficient to reproduce the translations on Windows. They contain only synthetic, independently created test input. No game files are required.

To reproduce, use a Windows x64 Rust toolchain with a functioning linker and C build tools, Git, PowerShell 7 and the official 64-bit dxcompiler.dll and dxil.dll from DirectXShaderCompiler release 1.9.2609.5. From the extracted directory run:

    pwsh -NoProfile -File .\run.ps1 -DxcDirectory 'C:\path\to\dxc\bin\x64'

The runner creates its own new directory, fetches exact public source commits and runs each probe with its supplied Cargo.lock. For a mismatch it prints match=false and retains the disassembly. An error compiling, validating or identifying one atomic aborts the run. This comparison was run using Rust 1.100.0 x86_64-pc-windows-gnu with GNU dlltool and LLVM-MinGW for C dependencies, on Windows, with DXC/dxil 1.9.2609.5. There was no GPU dispatch or title execution for these results. No inference about gameplay or the numeric resulting texture value is part of this measurement.

Public sources:
https://github.com/xcl-dev/xcl/commit/80f4819098e921f2e1ab371144d4832f68ffedd6
https://github.com/xcl-dev/xcl/pull/1948
https://github.com/xcl-dev/xcl/pull/1541
https://github.com/xcl-dev/xcl/issues/1485

As of the source review, #1948 showed test runs of direct_image_atomic_64bit_validates and a smoke test of the default HLSL path. No numeric readback for the actual direct backend's 64-bit UMax texture operation was found in that PR's test or its two discussion comments. #1541 reports runtime GPU testing of its separate HLSL implementation.
