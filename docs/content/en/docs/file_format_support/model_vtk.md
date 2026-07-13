---
date: '2026-06-12T15:54:46+01:00'
title: 'VTK/VTU'
category: 'Mesh Formats'
weight: 204
---

## Basics

- **Nodes**: POINTS section with coordinates
- **Elements**: Tetrahedral and hexahedral 8 and Wedge 6 cell types
- **Materials**: Cell data arrays for material properties
- **Density Mapping**: Element and node-based scalar fields for density

## Implementation Details

VTK files are supported as model formats through the [vtkio](https://docs.rs/vtkio/latest/vtkio/) crate, which provides a Rust interface for reading and writing VTK files. 
The library supports both ASCII and binary formats, as well as various cell types and data arrays.