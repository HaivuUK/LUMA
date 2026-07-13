//! NRRD (Nearly Raw Raster Data) reader and writer.
// Implementation of the NRRD image format just reading
// Taking inspirations from the following repositories:
// https://github.com/henriksson-lab/bioformats-rs/blob/main/src/formats/nrrd.rs
// https://github.com/mgevaert/nrrd/blob/main/nrrd/src/lib.rs
//! Specification: http://teem.sourceforge.net/nrrd/format.html
//! Supports inline (`.nrrd`) and detached (`.nhdr` + data file) formats.

use std::collections::HashMap;
use anyhow::{Result, anyhow, bail};
use nifti::Endianness;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use flate2::read::MultiGzDecoder;
use bzip2::read::BzDecoder;
use rayon::prelude::*;

use super::{Volume, PixelType};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModuloAnnotation {
	/// Which parent dimension this modulo subdivides: "Z", "C", or "T".
	pub parent_dimension: String,
	/// Type of sub-dimension: "lifetime", "lambda", "angle", "phase", "tile", or custom.
	pub modulo_type: String,
	/// Start value of the sub-dimension range.
	pub start: f64,
	/// Step size between consecutive sub-dimension values.
	pub step: f64,
	/// End value of the sub-dimension range.
	pub end: f64,
	/// Unit of the sub-dimension values (e.g., "nm", "ps", "degree").
	pub unit: String,
	/// Optional labels for each sub-dimension position.
	pub labels: Vec<String>,
}

/// Dimension ordering of the image planes.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DimensionOrder {
	XYCTZ,
	#[default] XYCZT,
	XYTCZ,
	XYTZC,
	XYZCT,
	XYZTC,
}

/// A typed metadata value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MetadataValue {
	String(String),
	Int(i64),
	Float(f64),
	Bool(bool),
	Bytes(Vec<u8>),
}

impl std::fmt::Display for MetadataValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MetadataValue::String(s) => write!(f, "{}", s),
			MetadataValue::Int(i) => write!(f, "{}", i),
			MetadataValue::Float(v) => write!(f, "{}", v),
			MetadataValue::Bool(b) => write!(f, "{}", b),
			MetadataValue::Bytes(b) => write!(f, "<{} bytes>", b.len()),
		}
	}
}

/// Optional indexed colour lookup table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LookupTable {
	pub red: Vec<u16>,
	pub green: Vec<u16>,
	pub blue: Vec<u16>,
}

/// Core metadata for one image series.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageMetadata {
	pub size_x: u32,
	pub size_y: u32,
	pub size_z: u32,
	pub size_c: u32,
	pub size_t: u32,
	pub pixel_type: PixelType,
	pub bits_per_pixel: u8,
	pub image_count: u32,
	pub dimension_order: DimensionOrder,
	pub is_rgb: bool,
	pub is_interleaved: bool,
	pub is_indexed: bool,
	pub is_little_endian: bool,
	pub resolution_count: u32,
	/// True if this series is a low-resolution thumbnail/preview rather than a
	/// full-resolution image.
	pub thumbnail: bool,
	pub series_metadata: HashMap<String, MetadataValue>,
	pub lookup_table: Option<LookupTable>,
	/// Modulo annotation for Z dimension (sub-dimensions within Z).
	pub modulo_z: Option<ModuloAnnotation>,
	/// Modulo annotation for C dimension (sub-dimensions within C).
	pub modulo_c: Option<ModuloAnnotation>,
	/// Modulo annotation for T dimension (sub-dimensions within T).
	pub modulo_t: Option<ModuloAnnotation>,
}

impl Default for ImageMetadata {
	fn default() -> Self {
		ImageMetadata {
			size_x: 0,
			size_y: 0,
			size_z: 1,
			size_c: 1,
			size_t: 1,
			pixel_type: PixelType::Uint8,
			bits_per_pixel: 8,
			image_count: 1,
			dimension_order: DimensionOrder::XYCZT,
			is_rgb: false,
			is_interleaved: false,
			is_indexed: false,
			is_little_endian: true,
			resolution_count: 1,
			thumbnail: false,
			series_metadata: HashMap::new(),
			lookup_table: None,
			modulo_z: None,
			modulo_c: None,
			modulo_t: None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Encoding {
    Raw,
    Gzip,
    Ascii,
    Bzip2,
    Unsupported,
}

#[derive(Debug)]
struct NrrdHeader {
    pixel_type: PixelType,
    dimension: usize,
    sizes: Vec<u32>,
    kinds: Vec<String>,
    space_directions: Vec<String>,
    space_origin: Option<String>,
    pixel_sizes: Vec<String>,
    pixel_size_units: Vec<String>,
    endian: Endianness,
    encoding: Encoding,
    data_file: Option<PathBuf>,
    data_files: Vec<PathBuf>,
    data_offset: u64,
    byte_skip: i64,
    line_skip: usize,
    extra: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct NrrdAxes {
    size_x: u32,
    size_y: u32,
    size_z: u32,
    size_c: u32,
    size_t: u32,
    axis_x: Option<usize>,
    axis_y: Option<usize>,
    axis_z: Option<usize>,
    axis_c: Option<usize>,
    axis_t: Option<usize>,
}

impl NrrdAxes {
    fn image_count(&self) -> u32 {
        self.size_z.max(1) * self.size_t.max(1)
    }
}

impl Default for NrrdHeader {
    fn default() -> Self {
        Self {
            pixel_type: PixelType::Uint8,
            dimension: 0usize,
            sizes: Vec::new(),
            kinds: Vec::new(),
            space_directions: Vec::new(),
            space_origin: None,
            pixel_sizes: Vec::new(),
            pixel_size_units: Vec::new(),
            endian: Endianness::Little,
            encoding: Encoding::Raw,
            data_file: None,
            data_files: Vec::new(),
            data_offset: 0u64,
            byte_skip: 0i64,
            line_skip: 0usize,
            extra: HashMap::new(),
        }
    }
}

impl Default for NrrdAxes {
    fn default() -> Self {
        Self {
            size_x: 1,
            size_y: 1,
            size_z: 1,
            size_c: 1,
            size_t: 1,
            axis_x: None,
            axis_y: None,
            axis_z: None,
            axis_c: None,
            axis_t: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edianness {
    Little,
    Big,
}

pub struct NrrdParser {
    path: PathBuf,
    meta: ImageMetadata,
    header: NrrdHeader,
}

pub fn load_nrrd(path: &str) -> Result<Volume> {
    let mut reader = NrrdParser::open(path)?;
    
    // Derive axes from the NRRD header and populate metadata sizes
    let axes = derive_nrrd_axes(&reader.header)?;
    reader.meta.size_x = axes.size_x;
    reader.meta.size_y = axes.size_y;
    reader.meta.size_z = axes.size_z;
    
    let volume = reader.load_volume()?;

    Ok(volume)
}

// Header fuctions

fn nrrd_data_path(parent: &Path, value: &str, detached_file: bool) -> Result<PathBuf> {
    let value = if detached_file {
        match value.find(['/', '\\']) {
            Some(i) => &value[i + 1..],
            None => value,
        }
    } else {
        value
    };
    let candidate = parent.join(value);
    
    // Try canonicalization first; if it fails (e.g., file doesn't exist yet),
    // return the relative path joined to parent and let callers handle it.
    let canon_parent = match parent.canonicalize() {
        Ok(p) => p,
        Err(_) => parent.to_path_buf(),
    };
    
    match candidate.canonicalize() {
        Ok(canon_candidate) => {
            if !canon_candidate.starts_with(&canon_parent) {
                bail!("NRRD detached file must stay within the header directory: {}", value);
            }
            Ok(canon_candidate)
        }
        Err(_) => {
            // File may not exist yet or canonicalize failed; return resolved path without canonicalization.
            // Use parent.join() to resolve .. and . components without requiring the file to exist.
            let resolved = canon_parent.join(value);
            Ok(resolved)
        }
    }
}

fn nrrd_pixel_type(ptype: &str) -> Result<PixelType> {
    let value = ptype.to_ascii_lowercase();
    match value.as_str() {
        "signed char" | "int8" | "int8_t" => Ok(PixelType::Int8),
        "uchar" | "unsigned char" | "uint8" | "uint8_t" => Ok(PixelType::Uint8),
        "short" | "short int" | "signed short" | "signed short int" | "int16" | "int16_t" => Ok(PixelType::Int16),
        "ushort" | "unsigned short" | "unsigned short int" | "uint16" | "uint16_t" => Ok(PixelType::Uint16),
        "int" | "signed int" | "int32" | "int32_t" => Ok(PixelType::Int32),
        "uint" | "unsigned int" | "uint32" | "uint32_t" => Ok(PixelType::Uint32),
        "float" => Ok(PixelType::Float32),
        "double" => Ok(PixelType::Float64),
        "block" => Ok(PixelType::Bit),
        _ => bail!("Unsupported pixel type: {}", ptype),
    }
}

fn parse_nrrd_header(path: &Path) -> Result<NrrdHeader> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut line = String::new(); // First line will be NRRD00XX
    reader.read_line(&mut line)?;
    if !line.trim_start().starts_with("NRRD") {
        return Err(anyhow!("Not a valid NRRD file: {}", path.display()));
    }
    line.clear();

    let mut header = NrrdHeader::default();

    let mut data_file_list = false;
    let parent = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();

    while reader.read_line(&mut line)? > 0 {
        let trimmed = line.trim_end_matches(['\r', '\n']);

        if trimmed.is_empty() {
            header.data_offset = reader.stream_position()?;
            break;
        }

        if data_file_list {
            header.data_files.push(nrrd_data_path(&parent, trimmed, false)?);
        } else if !trimmed.starts_with('#')
            && let Some((key, value)) = trimmed.split_once(":") {
                let key = key.trim().to_ascii_lowercase();
                let value = value.trim_start_matches(['=', ' ']).trim();

                match key.as_str() {
                    "type" => header.pixel_type = nrrd_pixel_type(value)?,
                    "dimension" => header.dimension = value.parse()?,
                    "sizes" => {
                        header.sizes = value
                            .split_ascii_whitespace()
                            .map(str::parse)
                            .collect::<Result<Vec<u32>, _>>()?;
                    }
                    "kinds" => {
                        header.kinds = value
                            .split_ascii_whitespace()
                            .map(|s| s.to_ascii_lowercase())
                            .collect();
                    }
                    "space directions" | "spacedirections" => {
                        header.pixel_sizes = value.split_ascii_whitespace().map(String::from).collect();
                        header.space_directions = value
                            .split_ascii_whitespace()
                            .map(|s| s.to_string())
                            .collect();
                    }
                    "space origin" | "spaceorigin" => {
                        header.space_origin = Some(value.trim_matches('"').to_string());
                    }
                    "spacings" => {
                        header.pixel_sizes = value.split_ascii_whitespace().map(String::from).collect();
                    }
                    "space units" | "spaceunits" => {
                        header.pixel_sizes = value
                            .split_ascii_whitespace()
                            .map(|s| s.trim_matches('"').to_string())
                            .collect();
                    }
                    "endian" => header.endian = if value.eq_ignore_ascii_case("little") {
                        Endianness::Little
                    } else {
                        Endianness::Big
                    },
                    "encoding" => {
                        header.encoding = match value.to_ascii_lowercase().as_str() {
                            "raw" => Encoding::Raw,
                            "gzip" | "gz" => Encoding::Gzip,
                            "ascii" | "text" | "txt" => Encoding::Ascii,
                            "bzip2" | "bz2" => Encoding::Bzip2,
                            _ => Encoding::Unsupported,
                        };
                    }
                    "data file" | "datafile" => {
                        if value.eq_ignore_ascii_case("LIST") {
                            data_file_list = true;
                        } else {
                            header.data_file = Some(nrrd_data_path(&parent, value, true)?);
                        }
                    }
                    "byte skip" | "byteskip" => header.byte_skip = value.parse()?,
                    "line skip" | "lineskip" => header.line_skip = value.parse()?,
                    _ => {
                        header.extra.insert(key.to_string(), value.to_string());
                    }
                }
            }
        line.clear();
    }
    Ok(header)
}

// Dimension order is always XYCZT
fn derive_nrrd_axes(header: &NrrdHeader) -> Result<NrrdAxes> {
    let mut axes = NrrdAxes::default();
    let mut space_dims = Vec::new();
    
    for (i, &size) in header.sizes.iter().enumerate() {
        let kind = header.kinds.get(i).map(|s| s.as_str()).unwrap_or("");
        match kind {
            "3d-color" | "rgb-color" | "vector" | "rgba-color" | "list" => {
                axes.size_c = size;
                axes.axis_c = Some(i);
            },
            "time" | "3d-time" => {
                axes.size_t = size;
                axes.axis_t = Some(i);
            },
            _ => {
                // Fallback heuristic if kinds are empty
                if space_dims.is_empty() && kind.is_empty() && (2..=16).contains(&size) {
                    axes.size_c = size;
                    axes.axis_c = Some(i);
                } else {
                    space_dims.push((i, size));
                }
            }
        }
    }

    if !space_dims.is_empty() { axes.size_x = space_dims[0].1; axes.axis_x = Some(space_dims[0].0); } // same as space_dims.len() > 0
    if space_dims.len() > 1 { axes.size_y = space_dims[1].1; axes.axis_y = Some(space_dims[1].0); }
    if space_dims.len() > 2 { axes.size_z = space_dims[2].1; axes.axis_z = Some(space_dims[2].0); }
    
    Ok(axes)
}

fn total_samples(sizes: &[u32]) -> Result<usize> {
    sizes.iter().try_fold(1usize, |acc, size| {
        acc.checked_mul(*size as usize)
            .ok_or_else(|| anyhow!("Overflow when calculating total samples"))
    })
}

fn nrrd_pixel_size(value: &str, index: usize) -> Option<f64> {
    let size = value.trim();
    if size.starts_with('(') && size.ends_with(')') {
        let vector = &size[1..size.len() - 1];
        return vector.split(',').nth(index).and_then(|v| v.trim().parse::<f64>().ok());
    }
    size.parse::<f64>().ok()
}

fn data_offset(base_offset: u64, header: &NrrdHeader, has_external_data: bool) -> Result<u64> {
    let mut offset = if has_external_data { 0 } else { base_offset };

    if header.byte_skip > 0 {
        offset += header.byte_skip as u64;
    } 
    // If byte_skip is -1 (or 0), we do nothing.
    // For inline files, it stays at base_offset.
    // For detached files, it stays at 0.

    Ok(offset)
}

// File reader functions

impl NrrdParser {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let header = parse_nrrd_header(&path)?;
        let mut meta = ImageMetadata {
            pixel_type: header.pixel_type,
            is_little_endian: header.endian == Endianness::Little,
            bits_per_pixel: (header.pixel_type.bytes_per_sample() * 8) as u8,
            ..Default::default()
        };

        Ok(Self { path, header, meta })
    }

    fn read_data(&self, expected_bytes: usize) -> Result<Vec<u8>> {
        let header = &self.header;
        
        let data_sources: Vec<(PathBuf, u64)> = if header.data_files.is_empty() {
            vec![(
                header.data_file.clone().unwrap_or_else(|| self.path.clone()),
                if header.data_file.is_some() { 0 } else { header.data_offset },
            )]
        } else {
            header.data_files.iter().map(|p| (p.clone(), 0)).collect()
        };

        let has_external_data = header.data_file.is_some() || !header.data_files.is_empty();
        let mut all = Vec::with_capacity(expected_bytes);

        for (data_path, base_offset) in &data_sources {
            let remaining = expected_bytes.saturating_sub(all.len());
            if remaining == 0 { break; }
            let mut chunk = self.read_nrrd_payload(data_path, *base_offset, header, remaining, has_external_data)?;
            all.append(&mut chunk);
        }

        if all.len() < expected_bytes {
            bail!("NRRD: data is shorter than expected. Got {}, expected {}", all.len(), expected_bytes);
        }
        Ok(all)
    }

    fn read_nrrd_payload(&self, data_path: &Path, base_offset: u64, header: &NrrdHeader, max_bytes: usize, has_external_data: bool) -> Result<Vec<u8>> {
        let mut file = File::open(data_path)?;
        let data_start = data_offset(base_offset, header, has_external_data)?;

        let data = match header.encoding {
            Encoding::Raw => {
                file.seek(SeekFrom::Start(data_start))?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                buf.truncate(max_bytes);
                buf
            }
            Encoding::Gzip => {
                file.seek(SeekFrom::Start(data_start))?;
                let mut dec = MultiGzDecoder::new(file);
                let mut all = Vec::new();
                dec.read_to_end(&mut all)?;
                all.truncate(max_bytes);
                all
            }
            Encoding::Ascii => {
                file.seek(SeekFrom::Start(data_start))?;
                let reader = BufReader::new(file);
                                
                let bps = self.meta.pixel_type.bytes_per_sample();
                let samples = max_bytes / bps.max(1);
                let mut buf = Vec::with_capacity(max_bytes);

                let tokens = reader.lines().map_while(Result::ok).flat_map(|line| {
                    let tokens_in_line: Vec<_> = line.split_whitespace().map(String::from).collect();
                    tokens_in_line
                });
                
                for token in tokens.take(samples) {
                    match self.meta.pixel_type {
                        PixelType::Uint8 | PixelType::Int8 => {
                            let v = token.parse::<u8>().map_err(|_| {
                                anyhow!("NRRD: malformed ASCII sample")
                            })?;
                            buf.push(v);
                        }
                        PixelType::Uint16 | PixelType::Int16 => {
                            let v = token.parse::<u16>().map_err(|_| {
                                anyhow!("NRRD: malformed ASCII sample")
                            })?;
                            buf.extend_from_slice(&v.to_le_bytes());
                        }
                        PixelType::Uint32 | PixelType::Int32 => {
                            let v = token.parse::<u32>().map_err(|_| {
                                anyhow!("NRRD: malformed ASCII sample")
                            })?;
                            buf.extend_from_slice(&v.to_le_bytes());
                        }
                        PixelType::Float32 => {
                            let v = token.parse::<f32>().map_err(|_| {
                                anyhow!("NRRD: malformed ASCII sample")
                            })?;
                            buf.extend_from_slice(&v.to_le_bytes());
                        }
                        PixelType::Float64 => {
                            let v = token.parse::<f64>().map_err(|_| {
                                anyhow!("NRRD: malformed ASCII sample")
                            })?;
                            buf.extend_from_slice(&v.to_le_bytes());
                        }
                        PixelType::Bit => {
                            let v = token.parse::<u8>().map_err(|_| {
                                anyhow!("NRRD: malformed ASCII sample")
                            })?;
                            buf.push(v);
                        }
                    }
                }
                if buf.len() != max_bytes {
                    return Err(anyhow!(
                        "NRRD: ASCII data is shorter than expected",
                    ));
                }
                buf
            }
            Encoding::Bzip2 => {
                file.seek(SeekFrom::Start(data_start))?;
                let mut dec = BzDecoder::new(file);
                let mut all = Vec::new();
                dec.read_to_end(&mut all)?;
                all.truncate(max_bytes);
                all
                // return Err(anyhow!(
                //     "NRRD bzip2 encoding is not supported (requires a bzip2 decoder crate)",
                // ));
            }
            _ => bail!("NRRD: unsupported encoding: {:?}", header.encoding),
        };
        Ok(data)
    }

    pub fn load_volume(&mut self) -> Result<Volume> {
        let header = &self.header;
        let axes = derive_nrrd_axes(header)?;
        let origin = extract_origin(header.space_origin.as_ref());

        let directions = if !header.space_directions.is_empty() {
            &header.space_directions
        } else {
            &header.pixel_sizes
        };

        let mut spacing = [1.0, 1.0, 1.0];
        for (i, dir_str) in directions.iter().enumerate().take(3) {
            spacing[i] = extract_spacing(dir_str);
        }

        self.meta.size_x = axes.size_x;
        self.meta.size_y = axes.size_y;
        self.meta.size_z = axes.size_z;
        self.meta.size_c = axes.size_c;
        self.meta.size_t = axes.size_t;

        let bps = self.meta.pixel_type.bytes_per_sample();
        
        // Use total_samples to ensure we don't truncate files containing channels/time
        let expected_bytes = total_samples(&header.sizes)?
            .checked_mul(bps)
            .ok_or_else(|| anyhow!("NRRD: byte count overflow"))?;

        let raw_bytes = self.read_data(expected_bytes)?;
        let scalars = self.extract_f32_volume(&raw_bytes, expected_bytes, &axes)?;

        let x: Vec<f64> = (0..self.meta.size_x).map(|i| origin[0] + (i as f64) * spacing[0]).collect();
        let y: Vec<f64> = (0..self.meta.size_y).map(|i| origin[1] + (i as f64) * spacing[1]).collect();
        let z: Vec<f64> = (0..self.meta.size_z).map(|i| origin[2] + (i as f64) * spacing[2]).collect();

        Volume::new(x, y, z, scalars)
    }

    fn extract_f32_volume(&self, all: &[u8], expected_bytes: usize, axes: &NrrdAxes) -> Result<Vec<f32>> {
        let meta = &self.meta;
        let header = &self.header;
        let bps = meta.pixel_type.bytes_per_sample();
        
        let mut strides = vec![1usize; header.sizes.len()];
        for i in 1..header.sizes.len() {
            strides[i] = strides[i - 1] * (header.sizes[i - 1] as usize);
        }

        let stride_x = axes.axis_x.map(|a| strides[a]).unwrap_or(0);
        let stride_y = axes.axis_y.map(|a| strides[a]).unwrap_or(0);
        let stride_z = axes.axis_z.map(|a| strides[a]).unwrap_or(0);
        let stride_c = axes.axis_c.map(|a| strides[a]).unwrap_or(0);
        let stride_t = axes.axis_t.map(|a| strides[a]).unwrap_or(0);

        let total_out_voxels = expected_bytes / bps;
        let mut scalars = vec![0.0; total_out_voxels];
        
        // Pre-calculate dimension sizes to avoid branching inside the hot loop
        let size_c = axes.size_c.max(1) as usize;
        let size_x = axes.size_x.max(1) as usize;
        let size_y = axes.size_y.max(1) as usize;
        let size_z = axes.size_z.max(1) as usize;
        
        // Pre-calculate pitch block sizes for fast flat-to-3D index math
        let pitch_cx = size_c * size_x;
        let pitch_cxy = pitch_cx * size_y;
        let pitch_cxyz = pitch_cxy * size_z;

        // Extract immutable copies of these so the Rayon closure doesn't have to borrow `self`
        let pixel_type = meta.pixel_type;
        let is_little_endian = header.endian == Endianness::Little;

        // Process the array completely in parallel
        scalars.par_iter_mut().enumerate().for_each(|(dst_idx, out_scalar)| {
            // Map the flat 1D output index to 5D coordinates
            let c = dst_idx % size_c;
            let x = (dst_idx / size_c) % size_x;
            let y = (dst_idx / pitch_cx) % size_y;
            let z = (dst_idx / pitch_cxy) % size_z;
            let t = dst_idx / pitch_cxyz;

            // Compute the source index in the native file layout
            let sample_index = c * stride_c 
                             + x * stride_x 
                             + y * stride_y 
                             + z * stride_z 
                             + t * stride_t;
                             
            let src = sample_index * bps;
            
            if src + bps <= all.len() {
                // Since `bytes_to_single_f32` is stateless, we can safely call it concurrently.
                *out_scalar = Self::parse_bytes_to_f32(&all[src..src+bps], &pixel_type, is_little_endian);
            }
        });

        Ok(scalars)
    }

    #[inline(always)]
    fn parse_bytes_to_f32(chunk: &[u8], pixel_type: &PixelType, is_little_endian: bool) -> f32 {
        match pixel_type {
            PixelType::Uint8 => chunk[0] as f32,
            PixelType::Int8 => (chunk[0] as i8) as f32,
            PixelType::Uint16 => {
                let arr: [u8; 2] = chunk.try_into().unwrap();
                if is_little_endian { u16::from_le_bytes(arr) as f32 } else { u16::from_be_bytes(arr) as f32 }
            }
            PixelType::Int16 => {
                let arr: [u8; 2] = chunk.try_into().unwrap();
                if is_little_endian { i16::from_le_bytes(arr) as f32 } else { i16::from_be_bytes(arr) as f32 }
            }
            PixelType::Uint32 => {
                let arr: [u8; 4] = chunk.try_into().unwrap();
                if is_little_endian { u32::from_le_bytes(arr) as f32 } else { u32::from_be_bytes(arr) as f32 }
            }
            PixelType::Int32 => {
                let arr: [u8; 4] = chunk.try_into().unwrap();
                if is_little_endian { i32::from_le_bytes(arr) as f32 } else { i32::from_be_bytes(arr) as f32 }
            }
            PixelType::Float32 => {
                let arr: [u8; 4] = chunk.try_into().unwrap();
                if is_little_endian { f32::from_le_bytes(arr) } else { f32::from_be_bytes(arr) }
            }
            PixelType::Float64 => {
                let arr: [u8; 8] = chunk.try_into().unwrap();
                if is_little_endian { f64::from_le_bytes(arr) as f32 } else { f64::from_be_bytes(arr) as f32 }
            }
            PixelType::Bit => chunk[0] as f32,
        }
    }
}

// Extracts the vector magnitude to ensure spacing is always positive
fn extract_spacing(dir_str: &str) -> f64 {
    let cleaned = dir_str.trim_matches(|c: char| c == '(' || c == ')' || c == ' ' || c.is_ascii_alphabetic());
    if cleaned.is_empty() { return 1.0; }
    
    let mag_sq: f64 = cleaned.split(',')
        .filter_map(|s| s.parse::<f64>().ok())
        .map(|v| v * v)
        .sum();
        
    let mag = mag_sq.sqrt();
    if mag > 0.0 { mag } else { 1.0 }
}

// Safely extracts the (X, Y, Z) starting position
fn extract_origin(orig_str: Option<&String>) -> [f64; 3] {
    let mut origin = [0.0, 0.0, 0.0];
    if let Some(s) = orig_str {
        let cleaned = s.trim_matches(|c: char| c == '(' || c == ')' || c == ' ');
        let parts: Vec<f64> = cleaned.split(',').filter_map(|p| p.parse().ok()).collect();
        origin[..3.min(parts.len())].copy_from_slice(&parts[..3.min(parts.len())]);
    }
    origin
}