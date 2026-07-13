---
date: '2026-06-12T15:54:46+01:00'
title: 'Command Line Reference'
category: 'General Information'
weight: 102
---

### Core Options
- `-p, --params <FILE>` - Parameter file (.toml)
- `-c, --ct <PATH>` - CT data path (.vtk, .nii, .nrrd or DICOM directory)  
- `-m, --mesh <FILE>` - Mesh (Abaqus .inp, ANSYS .cdb, or VTK .vtk or .vtu)

### Mesh Transformation Options
- `--trans <X> <Y> <Z>` - Translate mesh in X Y Z directions (mm)
- `--rot <X> <Y> <Z>` - Rotate mesh around X Y Z axes (degrees)

### Visualisation Options
- `--visualise [MODE]` - Enable visualisation. With no mode, LUMA prefers `processed` when both CT and params are present, otherwise `material` when params are present, otherwise `align` when CT is present, otherwise `material`.
- `--visualise align` - Visualise mesh and CT alignment without material assignment
- `--visualise material` - Visualise an already processed mesh with material assignments
- `--visualise processed` - Run material assignment and then visualise the result
- `--viz-slices <NUM>` - Number of slices per axis (default: 10)
- `--viz-resolution <SIZE>` - Image resolution (default: 512)
- `--viz-export <DIR>` - Export images to directory
- `--viz-port <PORT>` - Web server port (default: auto)
- `--viz-no-browser` - Don't auto-open browser

### Histogram Options
- `--histogram` - Export material histogram (CSV + JSON)
- `--histogram-dir <DIR>` - Output directory for histogram exports

### Debug Options
- `--verbose` - Enable verbose output
- `--debug` - Enable debug logging (also via LUMA_DEBUG=1 env var)

### Usage Examples

#### Preview Alignment
```bash
# Check alignment before processing
luma --visualise align --ct femur_scan.vtk --mesh femur_mesh.inp --rot 90.0 0.0 -45.0
```

Alternatively there are tranformation controls in the pre assignment visualiser that can be used to determine exactly what values to use.

```bash
# Check alignment before processing
luma --visualise align --ct femur_scan.vtk --mesh femur_mesh.inp
```

#### High-Resolution Visualisation
```bash
# Process with detailed visualisation
luma --visualise processed --params bone_parameters.toml --ct ct_scan.vtk --mesh bone_mesh.inp --viz-resolution 1024 --viz-export ./bone_visualisation
```

#### Material Visualisation
```bash
# Visualise a mesh that has previously had materials assigned to it
luma --visualise material --mesh bone_mesh.cdb
```