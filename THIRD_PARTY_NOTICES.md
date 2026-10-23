# Third-party notices

This Odia normalizer is distributed under Apache-2.0. Third-party components
retain their own copyrights and license terms. The main LICENSE file and the
license copies bundled with the Python package must accompany redistribution.

## Source attribution

Modified source-coordinate mapping, public result records, and work-control
infrastructure retain attribution to the original Apache-2.0 contributors,
including erdemtuna and Canberk Aslan. The original contribution identified by
commit `0a5de68286cacc4f95c754f519899d15fc2c5e73` is acknowledged. Original
copyright and license rights are retained; no contributor endorsement is implied.
Odia recognition, numeral vocabulary, speech rendering, cleanup, and tests are
maintained as modified and newly authored components of this project.

## Locked Rust dependencies

The following inventory is derived from the current locked Cargo metadata and
includes workspace runtime, optional, development, build, and target-specific
dependencies. Feature and target selection determine what is used in a binary.
License expressions are reproduced from the packages' own metadata. Original
license text copies are retained under `bindings/python/licenses`.

| Package | Version | Declared license |
|---|---|---|
| autocfg | 1.5.1 | Apache-2.0 OR MIT |
| bit-set | 0.11.1 | Apache-2.0 OR MIT |
| bit-vec | 0.10.1 | Apache-2.0 OR MIT |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| borsh | 1.8.1 | MIT OR Apache-2.0 |
| borsh-derive | 1.8.1 | Apache-2.0 |
| bytes | 1.12.1 | MIT |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| cfg_aliases | 0.2.2 | MIT |
| chacha20 | 0.10.2 | MIT OR Apache-2.0 |
| core_detect | 1.0.0 | MIT/Apache-2.0 |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 |
| equivalent | 1.0.3 | Apache-2.0 OR MIT |
| errno | 0.3.14 | MIT OR Apache-2.0 |
| fastrand | 2.5.0 | Apache-2.0 OR MIT |
| fnv | 1.0.7 | Apache-2.0 / MIT |
| getrandom | 0.4.3 | MIT OR Apache-2.0 |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 |
| heck | 0.5.0 | MIT OR Apache-2.0 |
| indexmap | 2.14.2 | Apache-2.0 OR MIT |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| libc | 0.2.189 | MIT OR Apache-2.0 |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| memchr | 2.8.3 | Unlicense OR MIT |
| num-traits | 0.2.19 | MIT OR Apache-2.0 |
| once_cell | 1.21.4 | MIT OR Apache-2.0 |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| proptest | 1.12.0 | MIT OR Apache-2.0 |
| pyo3 | 0.29.3 | MIT OR Apache-2.0 |
| pyo3-build-config | 0.29.3 | MIT OR Apache-2.0 |
| pyo3-ffi | 0.29.3 | MIT OR Apache-2.0 |
| pyo3-macros | 0.29.3 | MIT OR Apache-2.0 |
| pyo3-macros-backend | 0.29.3 | MIT OR Apache-2.0 |
| quick-error | 1.2.3 | MIT/Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| rand | 0.10.3 | MIT OR Apache-2.0 |
| rand_core | 0.10.1 | MIT OR Apache-2.0 |
| rand_xorshift | 0.5.0 | MIT OR Apache-2.0 |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 |
| rustix | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| rusty-fork | 0.3.1 | MIT/Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.6 | MIT OR Apache-2.0 |
| target-lexicon | 0.13.5 | Apache-2.0 WITH LLVM-exception |
| tempfile | 3.27.0 | MIT OR Apache-2.0 |
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_edit | 0.25.17+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 |
| unarray | 0.1.4 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| unicode-normalization | 0.1.25 | MIT OR Apache-2.0 |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 |
| wait-timeout | 0.2.1 | MIT/Apache-2.0 |
| windows-link | 0.2.1 | MIT OR Apache-2.0 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 |
| winnow | 1.0.4 | MIT |
| zmij | 1.0.23 | MIT |

The native Python binding uses PyO3; maturin is build tooling. Python consumers
need no additional runtime Python dependencies. Compiled Rust standard-library
notices are retained under `bindings/python/licenses/rust-std-1.99.0`.

## Odia Unicode / CLDR reference

Gregorian month names were cross-checked against Unicode CLDR release-48
`common/main/or.xml` (Copyright © 1991–2025 Unicode, Inc.). CLDR is a locale
reference, not a bundled runtime grammar. Source/data provenance and linguistic
review status are documented in `src/data/provenance.md`. Retain the following
notice for the referenced data and Unicode-licensed dependency data.

UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 1991-2026 Unicode, Inc.

NOTICE TO USER: Carefully read the following legal agreement. BY
DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

Permission is hereby granted, free of charge, to any person obtaining a
copy of data files and any associated documentation (the "Data Files") or
software and any associated documentation (the "Software") to deal in the
Data Files or Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, and/or sell
copies of the Data Files or Software, and to permit persons to whom the
Data Files or Software are furnished to do so, provided that either (a)
this copyright and permission notice appear with all copies of the Data
Files or Software, or (b) this copyright and permission notice appear in
associated Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
THIRD PARTY RIGHTS.

IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or other
dealings in these Data Files or Software without prior written
authorization of the copyright holder.
