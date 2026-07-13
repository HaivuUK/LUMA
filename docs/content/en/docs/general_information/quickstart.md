---
date: '2026-06-12T15:54:46+01:00'
title: 'Quick Start'
category: 'General Information'
weight: 101
---

# Basic Material Assignment
```bash
luma --params example_parameters.toml --ct example_ct_data.vtk --mesh example_mesh.inp
```

# 3D Visualisation
```bash
luma --visualise align --ct scan.vtk --mesh bone.inp
```

# Material Assignment Visualisation
```bash
luma --visualise processed --params parameters.toml --ct scan.vtk --mesh bone.inp
```

# With Mesh Transformation
```bash
luma --params parameters.toml --ct scan.vtk --mesh bone.cdb --rot 180.0 0.0 0.0 --trans 0.0 0.0 10.0
```
