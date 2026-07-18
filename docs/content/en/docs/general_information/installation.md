---
date: '2026-06-12T15:54:46+01:00'
title: 'Installation'
category: 'General Information'
weight: 100
---

>[!NOTE]
>LUMA is not currently code-signed, so Windows and MacOS may warn you about running the application.

### Easy install

#### Windows

The compiled installers are available in the [release](https://github.com/INSIGNEO/LUMA/releases/latest) section:
- `luma_X.X.X_x64-setup.exe` and `luma_X.X.X_x64_en-US.msi` - Installers that adds the tool to your path.

#### MacOS

A packaged universal app is available in the [release](https://github.com/INSIGNEO/LUMA/releases/latest) section:
- `luma.pkg` - Installer that adds the tools to your path.

#### Linux

LUMA is set up to build `.rpm`, `.deb`, and `.appimage`.
These will not automatically install to path and require some user setup to use in this way.

##### AppImage

Begin by making the AppImage an executable.
You may want to change the name to make your life easier.

```bash
chmod +x luma.AppImage
```

###### User Install

Add to your local bin folder.

```bash
mkdir -p ~/.local/bin
```

```bash
mv luma.AppImage ~/.local/bin/LUMA
```

Add the directory to your PATH (if needed)
Use nano or your preferred equivalent.

```bash
nano ~/.bashrc
```

And add the following to your file

```bash
export PATH="$HOME/.local/bin:$PATH"
```

```bash
source ~/.bashrc
```

###### System Install

Move the file to your bin folder and check it is executable.

```bash
sudo mv luma.AppImage /usr/local/bin/LUMA
```

```bash
sudo chmod +x /usr/local/bin/LUMA
```

##### RPM and DEB

These should not (I think? I do not use these formats) require extra setup to become available on PATH.

###### DEB (Debian, Ubuntu, etc.)

```bash
sudo apt update
sudo apt install ./luma.deb
```

###### RPM (Fedora, RHEL, CentOS, etc.)

```bash
sudo dnf install ./luma.rpm
```

### Manual Build Commands

For users who want to build from source please refer to the [Building From Source](../../development/building_from_source) page for instructions on how to build LUMA from source code.
