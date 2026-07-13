---
date: '2026-06-12T15:54:46+01:00'
title: 'Features'
category: 'Extra Info'
weight: 600
---

### Integration Schemes
- **Centroid**: Single point at element center (fastest)
- **Gauss Quadrature**: 1, 4, or 10-point rules for tetrahedra (most accurate)
- **Dense Sampling**: Adaptive grid sampling (good for large elements)

### Integration Methods

Multiple integration schemes are offered:
- **Dense Stepping**: Uniform grid sampling within elements (default)
- **Voxel-Based**: Sample at voxel intersections (faster for large elements)

### Performance Optimisations

LUMA has undergone a lot of work to try and be efficient. If you find something that does not perform as well as you expect, or you think I have implimented something wrong, please let me know.

The current optimisation steps are:

- **Parallel material assignment**: Elements processed in parallel across CPU cores
- **Direct buffer writing**: Manual formatting bypasses standard library overhead
- **Pre-computed lookups**: Material ID vectors eliminate HashMap lookups
- **Large buffer batches**: 1MB+ string buffers minimise I/O operations
- **Batch processing**: 5000 elements per batch reduces syscall overhead
- **Single write operations**: Replace many small writes with bulk operations
- **Spatial acceleration**: Efficient voxel lookup with bounds checking
- **SIMD utilisation**: Vectorised mathematical operations where possible