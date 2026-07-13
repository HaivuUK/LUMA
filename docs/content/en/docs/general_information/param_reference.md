---
date: '2026-06-12T15:54:46+01:00'
draft: true
title: 'Parameter File Reference'
category: 'General Information'
weight: 103
---

LUMA uses TOML as the config file format since v0.2.0.

>

The config file must be valid TOML; see [https://toml.io](https://toml.io) for further details continue reading.

>

Parameter files use TOML with snake_case keys and `#` comments. Each top-level heading becomes a TOML table such as `[luma_options]`, `[ct_calibration_coefficients]`, `[ct_calibration_correction]`, and `[modulus_calculation]`. Set exactly one of `gap_value` or `num_materials`.

# Core Parameters

## Integration Settings
```toml
# Integration mode - determines what values are assigned
[luma_options]
integration = "E"          # Options: "E" (modulus), "HU" (Hounsfield), "None" (density)

# Performance vs accuracy tradeoffs
integration_scheme = "dense" # Options: "dense", "voxel"
int_steps = 8                # Integration steps (density sampling resolution)
```

## Material Grouping
```toml
# How to group similar materials
gap_value = 50            # Minimum difference to create separate materials
# OR choose a fixed number of materials:
# num_materials = 12
grouping_density = "mean" # Options: "mean", "max" (how to combine element values)
min_val = 0.000001       # Minimum value clamp (prevents division by zero)
```

## Mesh Processing
```toml
# Parts/regions to ignore during processing
ignore = ["ACL", "pin"] # List of part names

# Poisson's ratio for all bone materials
poisson = 0.3
```

# CT Calibration Parameters

## Primary Calibration (HU to QCT Density)
```toml
# rhoQCT = rhoQCTa + (rhoQCTb * HU)
[ct_calibration_coefficients]
rho_qct_a = -0.01222      # Intercept coefficient
rho_qct_b = 0.0007079     # Slope coefficient
```

## Calibration Correction (QCT to Ash Density)
```toml
# Whether to apply secondary calibration correction
[ct_calibration_correction]
calibration_correct = true  # Options: true, false

# Single or multiple threshold calibration
num_ct_param = "single"     # Options: "single", "triple"

# For single mode
rho_asha1 = 0.07895      # First ash density intercept
rho_ashb1 = 0.8772       # First ash density slope

# For triple mode (different equations for different density ranges)
rho_thresh1 = 0          # Lower threshold
rho_thresh2 = 5          # Upper threshold
rho_asha1 = 0.07895      # Low density range intercept
rho_ashb1 = 0.8772       # Low density range slope
rho_asha2 = 0            # Medium density range intercept
rho_ashb2 = 1            # Medium density range slope
rho_asha3 = 0            # High density range intercept
rho_ashb3 = 1            # High density range slope
```

# Modulus Calculation Parameters

## Density to Modulus Conversion
```toml
# E = Ea + (Eb * RhoAsh)^Ec
[modulus_calculation]
num_e_param = "single"      # Options: "single", "triple"

# For single mode - one equation for all densities
ea1 = 0                 # Modulus intercept (MPa)
eb1 = 14664             # Modulus coefficient
ec1 = 1.49              # Modulus exponent

# For triple mode - different equations for different ranges
ethresh1 = 0            # Lower modulus threshold
ethresh2 = 0            # Upper modulus threshold
ea1 = 0                 # Low range intercept
eb1 = 14664             # Low range coefficient
ec1 = 1.49              # Low range exponent
ea2 = 0                 # Medium range intercept
eb2 = 1                 # Medium range coefficient
ec2 = 1                 # Medium range exponent
ea3 = 0                 # High range intercept
eb3 = 1                 # High range coefficient
ec3 = 1                 # High range exponent
```

# Mesh Transformation Parameters

## Translation (in millimeters)
```toml
[mesh_transformation]
mesh_translate_x = 0.0    # Translate in X direction
mesh_translate_y = 0.0    # Translate in Y direction
mesh_translate_z = 0.0    # Translate in Z direction
```

## Rotation (in degrees)
```toml
[mesh_transformation]
mesh_rotate_x = 0.0       # Rotate around X axis
mesh_rotate_y = 0.0       # Rotate around Y axis
mesh_rotate_z = 0.0       # Rotate around Z axis
```

**Note**: Command line transformation arguments take priority over parameter file settings.

# Histogram Parameters

```toml
[histogram]
histogram_export = true
histogram_dir = "./histograms"
```

# Parameter File Examples

## Fast Processing (Minimal Accuracy)
```toml
[luma_options]
integration = "E"
integration_scheme = "dense"
int_steps = 1
gap_value = 50
grouping_density = "mean"
min_val = 0.000001
poisson = 0.3

[ct_calibration_coefficients]
rho_qct_a = -0.01222
rho_qct_b = 0.0007079

[ct_calibration_correction]
calibration_correct = true
num_ct_param = "single"
rho_asha1 = 0.07895
rho_ashb1 = 0.8772

[modulus_calculation]
num_e_param = "single"
ea1 = 0
eb1 = 14664
ec1 = 1.49
```

## High Accuracy Processing
```toml
[luma_options]
integration = "E"
integration_scheme = "dense"
int_steps = 8             # Increased sampling density
gap_value = 50
grouping_density = "mean"
back_calculation = true
min_val = 0.000001
poisson = 0.3

[ct_calibration_coefficients]
rho_qct_a = -0.01222
rho_qct_b = 0.0007079

[ct_calibration_correction]
calibration_correct = true
num_ct_param = "single"
rho_asha1 = 0.07895
rho_ashb1 = 0.8772

[modulus_calculation]
num_e_param = "single"
ea1 = 0
eb1 = 14664
ec1 = 1.49
```

## With Mesh Transformation
```toml
# Standard parameters...
[luma_options]
integration = "E"
integration_scheme = "dense"
int_steps = 8
# ... (calibration parameters)

# Alignment transformations
[mesh_transformation]
mesh_rotate_x = 180.0     # Flip mesh upside down
mesh_translate_z = 50.0   # Move 50mm up in Z direction
```
