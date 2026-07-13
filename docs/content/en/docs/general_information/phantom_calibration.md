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

The generated values can be saved in to a TOML fil with extra information generated during the calibration. 
Currently, to allow for flexibility this will only output the calibration values and not all the values you need for a successful run.

Please refer to the [Parameter Reference](param_reference.md) section to see how to build a complete TOML file. 

>[!NOTE]
> LUMA should tell you if you are missing something from your file, however if it does not please file an issue so we can help.
