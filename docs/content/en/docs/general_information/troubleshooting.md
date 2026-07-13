---
date: '2026-06-12T15:54:46+01:00'
draft: true
title: 'Troubleshooting'
category: 'General Information'
weight: 105
---

# Common Issues

## Memory Issues
**Symptom**: Out of memory errors with large datasets
**Solutions**:
- Reduce `intSteps` parameter
- Increase system virtual memory
- Use 64-bit build for >4GB datasets

## Alignment Problems
**Symptom**: Material assignment looks incorrect in visualisation
**Solutions**:
- Use `--visualise align` to check mesh-CT alignment
- Adjust mesh transformation parameters
- Verify CT coordinate system and spacing
- Check mesh units (mm vs meters)

## Performance Issues
**Symptom**: Processing takes too long
**Solutions**:
- Reduce `intSteps` parameter
- Check CPU usage (should use all cores)

## Visualisation Problems
**Symptom**: Empty or incorrect visualisation
**Solutions**:
- Check WebGL and WebGPU support in browser
- Reduce `--viz-resolution` for large meshes

## File Format Issues
**Symptom**: Error reading mesh or CT files
**Solutions**:
- Verify file format (ASCII vs binary)
- Check file path and permissions
- Ensure supported element types
- Validate CT data format and units

# Debug Mode
Enable detailed logging:
```bash
# Via environment variable or build without the release tag
LUMA_DEBUG=1 ./target/release/luma --params params.toml --ct ct.vtk --mesh mesh.inp
```

```bash
# Via command line
luma --debug --params params.toml --ct ct.vtk --mesh mesh.inp
```

# Error Messages

## "No valid elements found"
- Check element type support (C3D4, C3D10, C3D8, C3D6)
- Verify mesh file format and parsing
- Check `ignore` parameter for excluded parts
- For visualisation this is a known issue if you have a full model

## "CT data outside mesh bounds"
- Verify mesh and CT coordinate systems
- Use mesh transformation to align data
- Check CT spacing and origin parameters

## "Integration failed"
- Reduce integration accuracy (lower `intSteps`)
- Try different integration scheme
- Check for degenerate elements in mesh