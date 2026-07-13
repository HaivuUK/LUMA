---
date: '2026-06-12T15:54:46+01:00'
draft: true
title: 'ANSYS CDB'
category: 'Mesh Formats'
weight: 203
---

## Basics

- **Nodes**: NBLOCK format
- **Elements**: EBLOCK format (all supported element types)
- **Materials**: MP and R command generation
- **Sections**: SECTYPE and SECDATA commands

## Implementation Details

ANSYS CDB files are handled in LUMA using a custom parser implemented in Rust. 
The parser reads the NBLOCK and EBLOCK sections to extract node and element data, respectively. 
Material properties are generated using the MP and R commands, while section definitions are created using the SECTYPE and SECDATA commands. 
The parser ensures that all supported element types are correctly interpreted and represented in the internal data structures of LUMA.