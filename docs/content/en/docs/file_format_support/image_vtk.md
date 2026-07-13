---
date: '2026-06-12T15:54:46+01:00'
draft: true
title: 'VTK'
category: 'Image Formats'
weight: 201
---

## Basics

- **ASCII format**: STRUCTURED_POINTS and RECTILINEAR_GRID
- **Binary format**: Automatic endianness detection
- **Multiple scalar types**: Float, double, integer data
- **Spacing and origin**: Automatic coordinate transformation

## Implementation Details

VTK files are supported as image formats through the [vtkio](https://docs.rs/vtkio/latest/vtkio/) crate, which provides a Rust interface for reading and writing VTK files. 
The library supports both ASCII and binary formats.
