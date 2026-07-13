---
date: '2026-06-12T15:54:46+01:00'
draft: true
title: 'BASH Batch Script'
category: 'Helpful Extras'
weight: 501
---

```powershell
# Process multiple datasets
$datasets = @("femur1", "femur2", "tibia1")
foreach ($dataset in $datasets) {
    .\<install path>\Users\<USERNAME>\AppData\Local\luma\luma.exe `
      --params "params\${dataset}_params.toml" `
      --ct "data\${dataset}.vtk" `
      --mesh "meshes\${dataset}.inp" `
}
```
