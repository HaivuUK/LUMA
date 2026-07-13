---
date: '2026-06-12T15:54:46+01:00'
title: 'LUMA Documentation'
weight: 100
---

LUMA is a high-performance Rust implementation bone material property assignment for finite element analysis.

This tool was developed in the [IMSB](https://www.sheffield.ac.uk/imsb) group at the [Insigneo Institute](https://www.sheffield.ac.uk/insigneo) at [the University of Sheffield](https://www.sheffield.ac.uk/), UK.
Please visit the [Insigneo Institute GitHub page](https://github.com/INSIGNEO) for more information on other projects and research.

>

The tool is heavily inspired by previous work done by:
- Elise Pegg and Richie Gill [<ref>[1](#ref1), [2](#ref2)</ref>].
- Istituto Ortopedico Rizzoli [<ref>[3](#ref3), [4](#ref4), [5](#ref5), [6](#ref6), [7](#ref7), [8](#ref8)</ref>].
    - [Bonemat](https://bonemat.ior.it/)
    - [ALBA](https://alba.ior.it/) (Agile Library for Biomedical Aplications)

### LUMA provides

- **High Performance**: Sub 2-minute processing on a 2 million line mesh file (in our testing, your milage may vary.)
- **3D Visualisation**: Interactive web-based material distribution viewer
- **Mesh Transformation**: Built-in rotation and translation for CT alignment
- **Advanced Integration**: Multiple integration schemes with adaptive accuracy
- **Wide Format Support**: Abaqus (.inp), ANSYS (.cdb), VTK, NRRD, NIfTI, and DICOM formats
- **Parallel Processing**: Multi-threaded material assignment with progress reporting

### Validation

For more information on the validation of LUMA, please see the [Validation](../about/validation/) page.

### Disclaimer

This software has been designed for research purposes only and has not been reviewed or approved by medical device 
regulation bodies.

This software is not to be used alone or in combination, for human beings for one or more of the following specific 
medical purposes:
- diagnosis, prevention, monitoring, prediction, prognosis, treatment or alleviation of disease.
- diagnosis, monitoring, treatment, alleviation of, or compensation for, an injury or disability.
- investigation, replacement or modification of the anatomy or of a physiological or pathological process or
state.
- providing information by means of _in vitro_ examination of specimens derived from the human body, including
organ, blood and tissue donations.

### License

AGPLv3 or later

Copyright (C) 2025  George Allison

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

### Credit

This project was inspired by [py_bonemat_abaqus](https://github.com/elisepegg/py_bonemat_abaqus) by [Elise Pegg](https://www.linkedin.com/in/elisepegg/) currently of Newcastle University and [Richie Gill](https://researchportal.bath.ac.uk/en/persons/richie-gill/) currently of University of Bath. And the work of [Istituto Ortopedico Rizzoli](https://www.ior.it/) in Bologna, Italy, particularly the work of the [Bonemat](https://bonemat.ior.it/) and [ALBA](https://alba.ior.it/) teams.

### References

<a id='ref1'>1</a>: Pegg EC, Gill HS.
py_bonemat_abaqus GitHub Repository.
(2016). [LINK](https://github.com/elisepegg/py_bonemat_abaqus)

<a id='ref2'>2</a>: Pegg EC, Gill HS. 
An open source software tool to assign the material properties of bone for ABAQUS finite element simulations.
J Biomechanics. 
In Press.
(2016). [DOI](http://dx.doi.org/10.1016/j.jbiomech.2016.07.037)

<a id='ref3'>3</a>: Schileo E, Pitocchi J, Falcinelli C, Taddei F.
Cortical bone mapping improves finite element strain prediction accuracy at the proximal femur.
Bone.
(2020). [DOI](https://doi.org/10.1016/j.bone.2020.115348)

<a id='ref4'>4</a>: Helgason B, Taddei F, Pálsson H,. et al.
A modified method for assigning material properties to FE models of bones.
Medical Engineering & Physics,
Volume 30, Issue 4, Pages 444-453.
(2008). [DOI](https://doi.org/10.1016/j.medengphy.2007.05.006) 

<a id='ref5'>5</a>: Taddei F, Schileo E, Helgason B,. et al.
The material mapping strategy influences the accuracy of CT-based finite element models of bones: An evaluation against experimental measurements.
Medical Engineering & Physics,
Volume 29, Issue 9, Pages 973-979.
(2007). [DOI](https://doi.org/10.1016/j.medengphy.2006.10.014) 

<a id='ref6'>6</a>: Taddei F, Pancanti A, Viceconti M.
An improved method for the automatic mapping of computed tomography numbers onto finite element models.
Medical Engineering & Physics,
Volume 26, Issue 1, Pages Pages 61-69.
(2004). [DOI](https://doi.org/10.1016/S1350-4533(03)00138-3) 

<a id='ref7'>7</a>: Zannoni C, Mantovani R, Viceconti M.
Material properties assignment to finite element models of bone structures: a new method.
Medical Engineering & Physics,
Volume 20, Issue 10, Pages 735-740.
(1999). [DOI](https://doi.org/10.1016/S1350-4533(98)00081-2) 

<a id='ref8'>8</a>: Crimi G, Vanella N, Schileo E, Valente G, Fraterrigo G, Taddei F.
ALBA: Agile library for biomedical applications.
SoftwareX,
Volume 31, Pages 102188.
(2025). [DOI](https://doi.org/10.1016/j.softx.2025.102188) 

<a id='ref9'>9</a>: Eggermont, Florieke and Verdonschot, Nico and van der Linden, Yvette and Tanck, Esther.
Calibration with or without phantom for fracture risk prediction in cancer patients with femoral bone metastases using CT-based finite element models.
PLOS ONE
Volume 14, Number 7.
(2019). [DOI](10.1371/journal.pone.0220564)