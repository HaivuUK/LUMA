---
title: "Graphical User Interface (GUI)"
description: "Support the ability to use a GUI for the software instead of the command line."
version: "0.X.0"
time: "Unknown"
---

### Plan

Currently LUMA is command line only.
With visualisation features being the only graphical aspect of the software.
Not everyone is comfortable with command line interfaces, and a GUI would make the software more accessible to a wider audience.

So the plan is to explore the possibility of a GUI for LUMA, ideally that does not require rewriting existing visualisation features, and can be integrated with the existing codebase.

#### Tauri

LUMA uses Tauri for the current visualisation features and the build pipeline, it would make sense to use Tauri for the GUI as well. 
However, I am bad at coding and do not know if the huge file size limitations and speed issues of those files are due to using Tauri and webview2 or if it is due to my own code. So I will need to explore this further.

#### Iced

There is no native 3d support in iced, so it would require a lot of work to implement functionality in iced that is already available in Tauri.
There is [ic3d](https://github.com/playtron-os/ic3d)

However, I have tried to use iced before and did not like it that much.
But again I am bad at coding, so I should look again.

#### egui

There is no native 3d support in egui, so it would require a lot of work to implement functionality in egui that is already available in Tauri.
However, there are crates that allow for 3d rendering in egui:

- [glow](https://github.com/grovesNL/glow) embeded in an eframe.

egui sitting on top of a 3D background:
- [bevy_egui](https://github.com/mvlabat/bevy_egui) 
- [three-d](https://github.com/asny/three-d)