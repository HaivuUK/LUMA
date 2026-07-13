---
date: '2026-06-12T15:54:46+01:00'
draft: true
title: 'Validation'
weight: 105
---

## Is LUMA Validated?

>[!Note]
>_(LUMA is in the process of being further validated, and we will update this page as more information becomes available.)_
>

>[!Note]
>I accidentally slept my remote workstation and cannot access the validation data for upload until I return to the office on the 20th of July
>


>

LUMA has been partially validated against other existing material mapping tools to ensure that it produces accurate and reliable results. 
The validation process involved comparing LUMA's outputs with those of established tools, as well as having tests in the codebase to assess its performance under various scenarios.

We have multiple test cases that cover a range of scenarios for comparison with other tools.
Using the [PyPeCT2S](https://github.com/HaivuUK/PyPeCT2S) pipeline to ensure consistent and repeatable finite element analysis, we have validated LUMA against the following tools:

- [Bonemat (build 1044 - 2026/05/20)](https://bonemat.ior.it/downloads)
- [py_bonemat_abaqus (version 1.0.9 - 2016/09/02)](https://github.com/elisepegg/py_bonemat_abaqus)

LUMA performs within 1% of the results produced by these tools.

>[!Note]
> This page is still a work in progress and will be updated as validation cases are completed.

<!-- We have provided the generated model files in a dedicated figshare repository for transparency and reproducibility. You can access the files [here](https://figshare.com/). -->

More information and graphs are available below.

### Validation Results

<!-- Graphs and detailed results will be added here -->

### Speed Tests

## Contributing to Validation

If you would like to help contribute to the validation of LUMA, please reach out to us via our [GitHub repository](https://github.com/HaivuUK/LUMA)