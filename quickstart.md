<h1 align="center">
  LUMA: Local Unit Modulus Assignment Quickstart Guide
</h1>

<p align="center" style="font-size: 16px;">
    Last updated for <b>LUMA 0.2.4</b>
</p>

<p align="center">
  <a href="https://github.com/INSIGNEO/LUMA/releases/latest"><del>Download</del></a> |
  <a href="https://insigneo.github.io/LUMA/"><del>Documentation</del></a> |
  <a href="mailto:GHAllison1@sheffield.ac.uk">Contact</a>
</p>

**Authors:** George H Allison<sup>1,2</sup>

**Affiliation:**

<sup>1</sup> [INSIGNEO Institute, University of Sheffield, United Kingdom](https://www.sheffield.ac.uk/insigneo);

<sup>2</sup> [School of Mechanical, Aerospace & Civil Engineering, University of Sheffield, United Kingdom](https://www.sheffield.ac.uk/mecheng)

**Contact:**
GHA ([Email](mailto:GHAllison1@sheffield.ac.uk) | [LinkedIn](https://www.linkedin.com/in/george-h-allison/)),
Dr X. Li ([Email](mailto:xinshan.li@sheffield.ac.uk))

A high-performance Rust implementation bone material property assignment for finite element analysis.
The tool is heavily inspired by previous work done by Elise Pegg and Richie Gill, 
and the work of the Bonemat team.

## Quickstart

Run the platform appropriate executable installer you have been provided,
or found in [releases](https://github.com/INSIGNEO/LUMA/releases/latest).

During install the executable will be added to your system path, so you can run it from any terminal.

### Version

```bash
luma --version
```

The LUMA version can be checked using the `--version` or `-V` flag.
This will print the version of LUMA that you have installed.
LUMA also prints the version in processed files, so you can see which version was used to process a given file.

### Help

```bash
luma --help
```
This will show you the help message, which includes the available options and their descriptions.

### Processing a file

```bash
luma --ct <input_file> --mesh <input_mesh> --params <input_params>
```

This will process the input file and generate an output file with assigned material properties.
Output files are saved in the same directory as the input file, with the same name and an added suffix `MAT`.
They will save in the same format as the input file, so if you input a `.vtk` file, the output will also be a `.vtk` file.

### Visualising

LUMA has a number of visualisation options which can be accessed using the `--visualise` flag.
The visualisation flag should pick the most appropriate type but can be overridden using a trailing argument,
in the form `--visualise [mode]`, where mode is one of `align`, `material`, or `processed`.

- `--visualise align` - Visualise mesh and CT alignment without material assignment
- `--visualise material` - Visualise an already processed mesh with material assignments
- `--visualise processed` - Run material assignment and then visualise the result

#### CT and Mesh alignment (align)

Requires CT and mesh files to be provided, and will visualise the alignment of the two files.
Transform controls are exposed in the visualisation window to see any adjustments that may be needed to align the two files.

#### Post-processing (processed)

Requires CT, mesh, and params files to be provided. 
This mode doesn't read the actual material properties from the mesh file, but visualises the properties directly in memory.
It can differ slightly from the output.

#### Previously processed files (material)

Requires a mesh file with material properties already assigned.
It will read material properties from the mesh file and visualise them.
It is capable of reading files generated in other software as long as they are valid file types.

#### Visualisation features

If you need an image of your visualised file you can right-click on the window and pick from one of the save options.
LUMA offers PNG and SVG output.

### Parameter file

LUMA uses a parameter file in TOML format to specify the material assignment parameters.

Parameter files use TOML with snake_case keys and `#` comments. 
Each top-level heading becomes a TOML table such as 
`[luma_options]`, 
`[ct_calibration_coefficients]`, 
`[ct_calibration_correction]`, 
and `[modulus_calculation]`. 
Set exactly one of `gap_value` or `num_materials`.

**Note**: Command line transformation arguments take priority over parameter file settings.

#### Core Parameters

##### Integration Settings
```toml
# Integration mode - determines what values are assigned
[luma_options]
integration = "E"          # Options: "E" (modulus), "HU" (Hounsfield), "None" (density)

# Performance vs accuracy tradeoffs
integration_scheme = "dense" # Options: "dense", "voxel"
int_steps = 8                # Integration steps (density sampling resolution)
```

##### Material Grouping
```toml
# How to group similar materials
gap_value = 50            # Minimum difference to create separate materials
# OR choose a fixed number of materials:
# num_materials = 12
grouping_density = "mean" # Options: "mean", "max" (how to combine element values)
min_val = 0.000001       # Minimum value clamp (prevents division by zero)
```

##### Mesh Processing
```toml
# Parts/regions to ignore during processing only for Abaqus .inp files
ignore = ["ACL", "pin"] # List of part names

# Poisson's ratio for all bone materials
poisson = 0.3
```

#### CT Calibration Parameters

##### Primary Calibration (HU to QCT Density)
```toml
# rhoQCT = rhoQCTa + (rhoQCTb * HU)
[ct_calibration_coefficients]
rho_qct_a = -0.01222      # Intercept coefficient
rho_qct_b = 0.0007079     # Slope coefficient
```

##### Calibration Correction (QCT to Ash Density)
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

#### Modulus Calculation Parameters

##### Density to Modulus Conversion
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

#### Mesh Transformation Parameters

##### Translation (in millimeters)
```toml
[mesh_transformation]
mesh_translate_x = 0.0    # Translate in X direction
mesh_translate_y = 0.0    # Translate in Y direction
mesh_translate_z = 0.0    # Translate in Z direction
```

##### Rotation (in degrees)
```toml
[mesh_transformation]
mesh_rotate_x = 0.0       # Rotate around X axis
mesh_rotate_y = 0.0       # Rotate around Y axis
mesh_rotate_z = 0.0       # Rotate around Z axis
```

#### Histogram Parameters

```toml
[histogram]
histogram_export = true
histogram_dir = "./histograms"
```

##### Example TOML
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

## Command Line Cheatsheet

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

## Disclaimer

This software has been designed for research purposes only and has not been reviewed or approved by medical device
regulation bodies.

This software is not to be used alone or in combination, for human beings for one or more of the following specific
medical purposes:
- diagnosis, prevention, monitoring, prediction, prognosis, treatment or alleviation of disease.
- diagnosis, monitoring, treatment, alleviation of, or compensation for, an injury or disability.
- investigation, replacement or modification of the anatomy or of a physiological or pathological process or
  state.
- providing information by means of _in vitro_ examination of specimens derived from the human body, including
  organ, blood and tissue donations.

## License

AGPLv3 or later

Copyright (C) 2025  George Allison

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
