---
date: '2026-06-12T15:54:46+01:00'
title: 'Known Issues'
category: 'Development'
weight: 400
---

## Unresolved

- Binning does not 1 to 1 match bonemat, however this may be based on differences in the gap value system. Explicitly setting the same number of materials should fix this, or help solve it. (Not sure if this is an issue or not) [0.X.X]
- Image export doesn't work/needs more control [0.X.X]
- SVG export of large models can cause OOM errors, need to find a better way to handle this. [=> 0.2.2]
- Webview2 based visualisation is currently very memory hungry. [=> 0.2.2]
- The _Z_ direction in views in CT views is not consistent. [=> 0.2.2]
- You need a mesh and CT files to use the phantom calibration tool, when you should only need a CT. [0.3.0]

## Resolved

### Fixed in 0.3.1
- The slider is an annoyingly imprecise way to move through the CT scans and having a way to move one at a time would be good. [=> 0.2.0]

### Fixed in 0.2.4
- Number of materials requested does not match the number in the file [0.2.3].

### Fixed in 0.2.3
- Resizing of the CT image appears to break LUMA until it is returned to its original size. [=> 0.2.2]
- Num of materials creates one less material than intended.
- Dicom based material determination fails to create material or sets incorrect materials values.

### Fixed in 0.2.1
- Issues with browser visualisation, visualisation may not load or takes long to load. (This issue has been seen on Zen[firefox based] but not clear if this is present on other browsers.)
- The current visualisation pipeline cannot seem to handle large models (possibly issue with the choice to use tiny-http to deliver the web ui, have begun exploring and trialing alternatives like [Tauri](https://v2.tauri.app/))
