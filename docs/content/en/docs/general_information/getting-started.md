---
date: '2026-06-12T15:54:46+01:00'
title: 'Getting Started'
category: 'General Information'
weight: 102
---

>[!INFO]
> There is currently no standalone GUI for the program.
> 
> Clicking the icon will flash a terminal window and then close it, which is not the intended way to run the program.

## How to use LUMA

Once the program is installed or built you can run it from the command line. 
The program is designed to be run from the command line, and it will provide you with a list of available commands and options.

When LUMA installs it should add itself to your system path, so you can run it from any directory.
If you have built the program from source, you will need to navigate to the directory where the executable is located.

1. Open a terminal or command prompt (e.g., PowerShell or Command Prompt on Windows, Terminal on macOS/Linux).
2. [OPTIONAL] If LUMA is not in path, navigate to the directory where the executable is located using the `cd` command, or write the entire path to LUMA.
3. Type `luma -h` or `luma --help` and press Enter. This will display the help message with a list of available commands and options.
4. You can also run LUMA with specific commands and options to perform tasks such as processing CT data, visualising meshes, or exporting results. For example, you can use the `--visualise` option to visualise a mesh and CT alignment.

Refer to the [Command Line Reference](./cli_reference.md) for a detailed list of commands and options available in LUMA.

## How to use the visualisation GUI

While LUMA does not have a standalone GUI for basic interaction, it does provide specific GUIs for targeted tasks.
LUMA has visualisation GUIs for the following tasks:

- Mesh and CT alignment
  - Transformation controls
  - Phantom calibration
- Material assignment visualisation
  - Basic histogram visualisation and export
  - Colourmaps with min and max controls
  - SVG export of visualisation and colourbar
  - PNG export of visualisation and colourbar

Visualisations can be triggered from the command line with the CLI option `--visualise` and this will spawn a visualisation window.
The visualisation modes in LUMA try and automatically determine the best mode to use based on the input files provided.
But you can also specify the visualisation mode with the `--visualise [MODE]` followed by the mode name.

Refer to the [Command Line Reference - Usage Examples](./cli_reference.md#usage-examples) for example of how to use the visualisation GUI.

## How to use Histograms

Histograms can be generated from the command line or parameter file.
And there is also a histogram visualisation in the `material` and `processed` visualisation modes.
This histogram is slightly more basic but tries to give enough control for basic needs.

If you want to generate histograms from the command line, you need to use the visualise option with the histogram option. 
The histogram will be generated and saved in the specified directory.
Here is an example command to generate histograms from a mesh file with materials:

```shell
luma --mesh mesh-file-with-materials.cdb --histogram --histogram-dir ./histogram-output --visualise
```

If you want to generate histograms from a parameter file, you need to specify the parameter file and the histogram option.
This requires you to run a command that requires the parameter file.
Here is an example command to generate histograms from a parameter file:

```toml
histogram_export = true
histogram_export_dir = "./histogram-output"
```