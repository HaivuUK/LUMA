---
title: "ESP/Phantom Calibration Feature"
description: "Provide options to perform phantom calibration within a CLI visualisation tool."
version: "0.3.0"
time: "q3 2026"
---

### Plan

LUMA already loads the CT scan for one of the visualisation GUI methods.
And the backend already has to pull the CT intensity values to complete its main function of assigning materials to the mesh.
So we can apply these same values to the phantom calibration process and do the equations for the user.

1. Provide basic box based ROI based phantom calibration that saves numbers to a TOML file.
2. Provide Cylindrical ROI based phantom calibration that saves numbers to a TOML file.
3. Provide a node based box ROI based phantom calibration that saves numbers to a TOML file.
4. Look at improving the accuracy and user-friendliness.

### Implementation

- [x] Provide basic box based ROI based phantom calibration that saves numbers to a TOML file. [added for 0.3.0]
- [x] Provide Cylindrical ROI based phantom calibration that saves numbers to a TOML file. [added for 0.3.7]
- [x] Provide a node based box ROI based phantom calibration that saves numbers to a TOML file. [added for 0.3.7]
- [ ] Look at improving the accuracy and user-friendliness.