---
date: '2026-06-12T15:54:46+01:00'
title: 'DICOM'
category: 'Image Formats'
weight: 200
---

## Basics

- **Multi-file series**: Automatic file discovery and sorting
- **Rescale intercept/slope**: Automatic HU value correction
- **Spacing extraction**: Voxel size from DICOM headers
- **3D volume reconstruction**: Slice ordering and coordinate mapping

## Implementation Details

Dicoms are handled in LUMA using the [dicom-rs](https://dicom-rs.github.io/) ecosystem.
This includes various crates for reading, writing, and manipulating DICOM files in Rust. The following crates are used:

[dicom-object](https://docs.rs/dicom-object/latest/dicom_object/)
[dicom-core](https://docs.rs/dicom-core/latest/dicom_core/)
[dicom-dictionary-std](https://docs.rs/dicom-dictionary-std/latest/dicom_dictionary_std/)
[dicom-encoding](https://docs.rs/dicom-encoding/latest/dicom_encoding/)
[dicom-pixeldata](https://docs.rs/dicom-pixeldata/latest/dicom_pixeldata/)