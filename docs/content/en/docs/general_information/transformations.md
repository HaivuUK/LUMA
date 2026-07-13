---
date: '2026-06-12T15:54:46+01:00'
title: 'Transformations'
category: 'General Information'
weight: 104
---

Transformations can be applied in LUMA through either the CLI flags or through the parameter file.

They are applied in a specific order and CLI flags will override the parameter file settings.

### Coordinate System
- **X-axis**: Left-right (positive = right)
- **Y-axis**: Anterior-posterior (positive = anterior/forward)  
- **Z-axis**: Superior-inferior (positive = superior/up)
- **Rotations**: Right-hand rule (positive = counterclockwise)

### Transformation Order
1. **Rotation first**: Applied around origin (X, then Y, then Z axes)
2. **Translation second**: Applied after rotation

### Usage Examples

**Command Line (Priority)**:
```bash
# Rotate 180° around X, translate 10mm in Z
luma -p params.toml -c ct.vtk -m mesh.cdb --rot 180.0 0.0 0.0 --trans 0.0 0.0 10.0
```

**Parameter File**:
```toml
[mesh_transformation]
mesh_rotate_x = 180.0
mesh_translate_z = 10.0
```

### Common Use Cases
- **Flip mesh upside down**: `--rot 180.0 0.0 0.0`
- **Rotate around vertical axis**: `--rot 0.0 0.0 90.0`
- **Align with CT coordinate system**: `--trans -50.0 -30.0 -100.0`
