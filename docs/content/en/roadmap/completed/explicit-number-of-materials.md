---
title: "Material Value Choices."
description: "Offer the choice to use either gap value or a fixed number of materials."
version: "0.2.0"
time: "Q2 2026"
---

### Plan 

Currently the number of materials is determined by the gap value, but it may be useful to have the option to specify a fixed number of materials instead.

### Implementation Details

Added the choice to use either a gap value or a fixed number of materials for material grouping, providing more flexibility in material assignment strategies.
The choice is mutally exclusive and only one can be specified in the config file.