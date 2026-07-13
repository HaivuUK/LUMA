---
title: "Additional Image Format Support"
description: "Support image standards NIFTI and NRRD."
version: "0.2.6"
time: "Q3 2026"
---

### Plan

- [x] Add support for NIFTI.
- [x] Add support for NRRD. 

#### NIfTI
Nifti can be supported through the `nifti` crate, which is a Rust library for reading and writing NIfTI files.

#### NRRD
NRRD does not have a rust crate, there is support in the bioformats crate but this is a full llm translation 
that is early stages and excessive for what is needed.
The NRRD format is open source and text based so a custom parser is possible, but will require some time to implement.

There are limitations on the the type of compression that can be supported.
The NRRD format supports raw, gzip, and bzip2 compression. The `flate2` crate can be used for gzip support, but
bzip2` is available through https://github.com/trifectatechfoundation/bzip2-rs libbz2 https://trifectatech.org/blog/bzip2-crate-switches-from-c-to-rust/
Need to explore more.

bzip2 mention that the are working on a rust port https://gitlab.com/bzip2/bzip2/ however it doesn't look like it is in active development and not available yet.

### Implementation

#### NIfTI
Nifti was supported through the `nifti` crate, which is a Rust library for reading and writing NIfTI files.

#### NRRD
NRRD was supported through a custom parser, based on the NRRD format specification and the `bioformats` crate implementation.
The NRRD format supports raw, gzip, and bzip2 compression. 
The `flate2` crate was used for gzip support.
The `bzip2` crate was used for bzip2 support (not sure if this works though as no test cases to hand).