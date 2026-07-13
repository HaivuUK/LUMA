---
date: '2026-06-12T15:54:46+01:00'
title: 'Changelog'
category: 'Development'
weight: 401
---

### Release Candidate 9 (formats, python bindings, and fixes) [September 2026] [0.4.X]
- TODO: Improve project structure and modularity, currently the code is quite monolithic and could be better organised into separate modules and files for better maintainability and readability.
- TODO: Add support for additional mesh formats such as Neutral (.ntr) files, ANSYS Input (.inp) files, FEBIO files, and MARCs files.
- TODO: Python bindings using PyO3 for drop-in replacement in existing Python workflows.

#### [0.3.X]

- TODO: Fix the CLI and TOML export of images.
- TODO: HTML performance improvements.
- TODO: Cannot visualise files that have unsupported element types in them.
- TODO: Test bonemat and luma at same integration value to see if that is the source of difference.
- TODO: Workout a better way to handle the SVG export of large models so that there are issues with OOM errors.
- TODO: Workout a method that produces leaner SVG files that don't have occluded geometry in them, which is currently a limitation of the SVG export.
- TODO: Look at how the abaqus sets are defined and the part definitions, setting global as part name from mesh/mod.rs is probably not the best approach.
- TODO: Improve the phantom calibration tool, currently it is a bit clunky and could be improved to be more user-friendly and intuitive.

#### [0.3.0] First Public Release

- Refactored the mesh code to make the abaqus file easier to manage.
- Refactored the volume code into separate modules to make it easier to manage and maintain.
- Expanded scan format support (NIfTI and NRRD).
- Clippy lint fixes and code refactoring to improve code quality and maintainability.
- Frontend cleanup and fixes.
- Changed abaqus files to not set part names to global, for single part files.
- Support FeBIO files.
- Phantom Calibration tool built in to the GUI.
- TODO: Clean up the codebase, remove unused code and dependencies, and improve documentation for better maintainability and usability.

### Release Candidate 8 (enhancement, features, and fixes) [May/June 2026] [0.2.0 - 0.2.5]

#### [0.2.5]

- Moved from JSON serialised IPC to Tauri raw IPC to improve visualisation performance.
- Made improvements to the RAM usage of the visualisation GUI, by using the built-in clipping planes in Three.js and better material use.
- Rotation in the visualisation window was originally setup with Three.js OrbitControls, which have a fundamental limitation on the rotation of the camera around the target point. This has been replaced with ArcballControls, which allows for a more natural and unrestricted rotation of the camera around the target point.
- Updated to use the latest version of Three.js (r185).
- Added the Three.js ViewHelper to the visualisation window, which allows for a better understanding of the orientation of the model in 3D space.
- Moved to HUGO for documentation, which allows for better organisation and navigation of the documentation, as well as better support for versioning and multilingual content.

#### [0.2.4]

- Some code refactoring and output clarity, printing the luma version used in generated files to create repeatablity.
- Visualisation is now a single consolidated command with granular control through subcommands.
- Added support for VTK file type import and export.
- New fix attempt for number of materials.
- Increased accuracy of abaqus inp file.
- Added density back calculation to the inp files.

#### [0.2.3]

- Fixed number of materials not matching the value set in the config.
- Fixed issue when using dicom files, where a double correction was being applied and skewing the outputted.
- Fix resizing of the CT image appears to break LUMA until it is returned to its original size/CT images don't resize and move with the window.

#### [0.2.2]

- Fix visualisation for abaqus files.
- Fix scanIP generated abaqus files.
- Add the mean value to the histogram.
- Improve the mesh + CT view. It looked bad and cluttered in comparison to the normal visualisation view.
- Consolidated the html viewer to be a single file correctly called in tauri.
- CT image picking from dicom stack.
- Better DICOM image support.

#### [0.2.1]

- Consolidate into one build type, don't need to separate feature builds for visualisation and non-visualisation, just have the visualisation features be optional and not used if not needed.
- Some minor fixes to visualisation performance (see [known issues](../known_issues)).
- Ported from `tiny_http` to [Tauri v2](https://v2.tauri.app/) to mitigate issues with visualisation (this presents the opportunity for a full GUI in the future alongside the CLI).

#### [0.2.0]

- Moved from legacy `key=value` parameter file format to structured TOML format for better readability and validation.
- Added the choice to use either a gap value or a fixed number of materials for material grouping, providing more flexibility in material assignment strategies.
- Histogram export functionality added, giving a csv and json file with the distribution of materials, with one command.
- Histogram view added to the visualisation page, with extensive options, and the ability to export the histogram as an image.
- Control over the visualisation histogram, such as the number of bins, the range, and the scale (linear or logarithmic).
- More control over visualisation page.
- Can now right-click save image (material distribution and colorbar) on the visualisation page to export the current view as an image (png and svg).
- Updated to use the latest version of THREE.js (r184).
- Migrated from THREE.js WebGL to WebGPU.
- Allow users to visualise models that have already been processed with material assignments, without needing to re-run the assignment process. This would allow users to quickly check the material distribution on their models without needing to wait for the assignment process to complete again.



### Release Candidate 1 - 7 (stability and core features) [October 2025] [0.1.X]
- Initial releases with core functionality and visualisation features.
- Used to test and refine against other workflows and users that may be interested in using the tool.