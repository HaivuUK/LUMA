---
title: "PVE Mitigation techniques"
description: "Improve the PVE implementation by using iterative segmentation and PVE correction."
version: "0.2.X"
time: "Q2 2026"
---

## Plan

Use iterative segmentation and PVE correction to improve the PVE implementation.
Ranniger and Schileo methodology.

1. Start with raw HU values.
2. Segment cortical bone using a threshold.
3. Compute bone fraction from the segmented mask vs voxel size.
4. Correct HU values using the bone fraction.
5. Iterate from step 2 until convergence.