---
date: '2026-06-12T15:54:46+01:00'
title: 'Phantom Calibration'
category: 'General Information'
weight: 108
---

LUMA supports phantom or phantomless manual calibration through its visualisation GUI.

>[!NOTE]
> This feature is new and the results have only been partially verified.

## Usage

>[!CAUTION]
> The GUI displays slope and intercept values as \( mg / {cm}^3 \) and \( HU \) respectively, to match the units of the European Spine Phantom.
>
> However, the values are stored in the TOML file as \( g / {cm}^3 \) and \( HU \) as this is the standard for LUMA.

>

The calibration feature in LUMA is available within the visualisation GUI through the `align` option.

This can be called through `--visualise` when only a CT option has been defined or `--visualise align` when you have defined all options.

In the visualisation GUI there is the option in the top menu bar list as `Calibration`.

### Adding ROIs

In the calibration window there is a button at the top of the panel with the button `Add ROI`.
When an ROI is added a cube will be added to the 3D view and squares added to each of the CT views.

The slice number in each plane can be changed to extend the box.
Additionally, the box can be moved in all the views, in 3D through the transform controls, and in the CT view through grabbing the middle.
In the CT view the size of the box can be altered through click and dragging on any of the edges.

### Calculating Slope and Intercept

LUMA needs a minimum of two different HU values to calculate the slope and intercept, however, it is strongly recommended to use at least 3.

A successful calculation will produce `Slope` and `Intercept` values at the bottom of the panel.

### Saving values

The generated values can be saved in to a TOML file with extra information generated during the calibration. 
Currently, to allow for flexibility this will only output the calibration values and not all the values you need for a successful run.

Please refer to the [Parameter Reference](param_reference.md) section to see how to build a complete TOML file. 

The extra information generated looks like this:

```toml
[ct_calibration_coefficients]
rho_qct_a = 0.0003452441
rho_qct_b = -0.0138929958

# Calculated using points: Base(HU:134->BMD:100), ROI 2(HU:-50->BMD:-50)
#roi_0 = { name = "ROI 1", x = [14, 15], y = [28, 29], z = [-119, -118] }
#roi_1 = { name = "ROI 2", x = [5, 15], y = [2, 12], z = [-516, -506] }
```

>[!NOTE]
> You should still try and make a valid TOML file with all the required parameters for a successful run, and use the calibration values in that file.
> 
> LUMA should tell you if you are missing something from your file, however if it does not please file an issue so we can help.
