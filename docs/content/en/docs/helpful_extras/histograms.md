---
date: '2026-06-12T15:54:46+01:00'
title: 'Histograms'
category: 'Helpful Extras'
weight: 502
---

LUMA is capable of generating histograms from the material assignment process.
This can be useful for quickly checking material distributions, identifying outliers, or for debugging purposes.

LUMA offers three types of histograms:
1. **JSON Histogram**: Outputs a JSON file containing the histogram data, which can be easily parsed and used in other applications.
2. **CSV Histogram**: Outputs a CSV file containing the histogram data, which can be opened in spreadsheet applications for further analysis.
3. **Interactive Histogram**: Generates an interactive HTML histogram in the visualisation window that allows users to inspect and explore in real-time.

## Usage

### JSON and CSV Histogram

To generate a JSON and CSV histogram, use the `--histogram` option, this can be followed by `--histogram-dir` to set the desired output file path. 
Otherwise, the histogram will be saved in the current working directory.

LUMA will generate both with the histogram command.

For example:

```bash
luma --params params.toml --ct image.vtk --mesh mesh.inp --histogram
```

In the case you forgot to grab a histogram during the initial material assignment, you can generate one by loading the mesh and calling the visualise and histogram command options:

```bash
luma --mesh mesh.inp --visualise --histogram
```

### Interactive Histogram

To generate an interactive histogram, use the `--visualise` option when you load an already assigned mesh, or in a processing run with the `--visualise` option.

Once the visualisation window is open, you can select the `Histogram` option from the top menu bar to view the interactive histogram.
This will pop up a floating window with an interactive histogram.

You can alter the following parameters in the histogram window:
- **Number of bins**: Adjust the number of bins in the histogram to change the granularity of the data representation.
- **X Range**: Set the range of values to be displayed on the x-axis.
- **Y Range**: Set the range of values to be displayed on the y-axis.
- **Y Scale**: Choose between linear and logarithmic scaling for the y-axis to better visualize data with large variations in frequency.

## Plotting

When you have the CSV or JSON histogram you can use your preferred plotting library to visualise the histogram data.
Below is an example of how to plot the data with Python.

### Raw Sample Reconstruction

```python
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Read CSV
df = pd.read_csv('histogram.csv')

smaple = np.repeat(df['value'].values, df['count'].astype(int).values)
plt.figure(figsize=(10, 6))
plt.hist(smaple, bins='auto', edgecolor='black')
plt.xlabel("Young's Modulus (E)")
plt.ylabel("Material Fequency")
plt.title("Histogram reconstructed from counts")
plt.tight_layout()
plt.show()
```

### Bin Centers

```python
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Read CSV
df = pd.read_csv('histogram.csv')

plt.figure(figsize=(10, 6))
plt.bar(df['value'], df['count'], width=(df['value'].diff().fillna(df['value'].iloc[1]-df['value'].iloc[0])).abs())
plt.xlabel("Young's Modulus (E)")
plt.ylabel("Material Fequency")
plt.title("Histogram bar plot using value vs. count")
plt.tight_layout()
plt.show()
```