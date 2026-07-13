---
date: '2026-06-12T15:54:46+01:00'
title: 'NRRD'
category: 'Image Formats'
weight: 203
---

## Implementation Details

>[!Note]
> Raw and gzip compressed files are confirmed to work.
> bzip2 compressed files are not confirmed to work, but the `bzip2` crate is used for decoding and may work.

NRRD files are supported as image formats through a custom parser, based on the NRRD format specification.
The library supports raw, gzip, and bzip2 compression.
LUMA supports loading NRRD files with these compression types.