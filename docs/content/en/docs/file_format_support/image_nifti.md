---
date: '2026-06-12T15:54:46+01:00'
draft: true
title: 'NIfTI'
category: 'Image Formats'
weight: 202
---

## Implementation Details

>[!Note]
> The NIfTI crate only support NIfTI-1 formats. 
> LUMA therefore does not support NIfTI-2 formats at this time.

NIfTI files are supported as image formats through the [nifti](https://docs.rs/nifti/latest/nifti/) crate, 
which provides a Rust interface for reading and writing NIfTI files. 
The library supports both ASCII and binary formats.
LUMA supports loading `nii` and `nii.gz` files.

NIfTI maps normally to a RAS (Right-Anterior-Superior) coordinate system, which is a common convention in neuroimaging, 
however LUMA uses the LPS (Left-Posterior-Superior) coordinate system.
So when loading NIfTI files, LUMA automatically converts the coordinates from RAS to LPS to maintain consistency with its internal representation.

```rust
let x: Vec<f64> = (0..nx).map(|i| -{affine[(0, 0)] * i as f64 + affine[(0, 3)]}).collect(); // Negative to convert from RAS to LPS
let y: Vec<f64> = (0..ny).map(|i| -{affine[(1, 1)] * i as f64 + affine[(1, 3)]}).collect(); // Negative to convert from RAS to LPS
let z: Vec<f64> = (0..nz).map(|i| affine[(2, 2)] * i as f64 + affine[(2, 3)]).collect();    // No change needed for Z-axis as it is the same in both RAS and LPS
```

We would recommend you check your model and image data to ensure that the coordinate system is correctly interpreted using the `--visualise` or `--visualise align` flags.