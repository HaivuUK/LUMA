use anyhow::{Result, anyhow};

use super::{Volume};

// Load a vtk file
pub fn load_vtk(path: &str) -> Result<Volume> {
    let data = std::fs::read(path)?;
    // Split header (ASCII) from data: header ends at first occurrence of '\nLOOKUP_TABLE' line + its newline.
    let marker = b"LOOKUP_TABLE";
    let mut header_end = None;
    for i in 0..data.len().saturating_sub(marker.len()) {
        if &data[i..i+marker.len()] == marker {
            // advance to end of line
            let mut j = i; while j < data.len() && data[j] != b'\n' { j+=1; }
            header_end = Some(j+1); break;
        }
    }
    let header_bytes = match header_end { Some(e)=> &data[..e], None => &data[..std::cmp::min(4096,data.len())] };
    let header = String::from_utf8_lossy(header_bytes);
    let upper = header.to_ascii_uppercase();
    if !upper.contains("DATASET") { return Err(anyhow!("VTK header missing DATASET")); }
    let dataset = if upper.contains("RECTILINEAR_GRID") { "RECTILINEAR_GRID" } else if upper.contains("STRUCTURED_POINTS") { "STRUCTURED_POINTS" } else { return Err(anyhow!("Unsupported DATASET (expect RECTILINEAR_GRID or STRUCTURED_POINTS)")); };
    // Common fields
    let mut dims = [0usize;3];
    if let Some(line) = header.lines().find(|l| l.to_ascii_uppercase().starts_with("DIMENSIONS")) {
        let toks: Vec<&str> = line.split_whitespace().collect(); if toks.len()>=4 { dims[0]=toks[1].parse()?; dims[1]=toks[2].parse()?; dims[2]=toks[3].parse()?; }
    }
    if dims.contains(&0) { return Err(anyhow!("DIMENSIONS not parsed")); }
    let mut origin = [0f64;3];
    if let Some(line) = header.lines().find(|l| l.to_ascii_uppercase().starts_with("ORIGIN")) { let toks: Vec<&str> = line.split_whitespace().collect(); if toks.len()>=4 { origin[0]=toks[1].parse()?; origin[1]=toks[2].parse()?; origin[2]=toks[3].parse()?; } }
    let mut spacing = [1f64;3];
    if let Some(line) = header.lines().find(|l| l.to_ascii_uppercase().starts_with("SPACING")) { let toks: Vec<&str> = line.split_whitespace().collect(); if toks.len()>=4 { spacing[0]=toks[1].parse()?; spacing[1]=toks[2].parse()?; spacing[2]=toks[3].parse()?; } }
    // POINT_DATA / SCALARS
    let mut npoints = dims[0]*dims[1]*dims[2];
    if let Some(line) = header.lines().find(|l| l.to_ascii_uppercase().starts_with("POINT_DATA")) { let toks: Vec<&str> = line.split_whitespace().collect(); if toks.len()>=2 { npoints = toks[1].parse()?; } }
    let mut scalar_type = "short".to_string();
    if let Some(line) = header.lines().find(|l| l.to_ascii_uppercase().starts_with("SCALARS")) { let toks: Vec<&str> = line.split_whitespace().collect(); if toks.len()>=3 { scalar_type = toks[2].to_ascii_lowercase(); } }
    // ASCII vs BINARY detection
    let is_ascii = upper.contains("ASCII\n") || upper.contains("ASCII\r\n");
    if is_ascii {
        if dataset == "RECTILINEAR_GRID" {
            // Re-run ASCII parser using existing logic but from header+remaining full text
            let full = String::from_utf8_lossy(&data);
            return load_vtk_ascii_rectilinear(&full);
        } else if dataset == "STRUCTURED_POINTS" {
            // ASCII structured points: scalars appear after LOOKUP_TABLE
            let full = String::from_utf8_lossy(&data);
            return load_vtk_ascii_structured_points(&full);
        }
    }
    // For STRUCTURED_POINTS (or RECTILINEAR_GRID binary unsupported currently) treat coordinates as implicit from origin+spacing
    if dataset == "RECTILINEAR_GRID" { return Err(anyhow!("Binary RECTILINEAR_GRID not yet supported")); }
    let start = header_end.ok_or_else(|| anyhow!("Could not locate start of binary scalar data (LOOKUP_TABLE line)"))?;
    let payload = &data[start..];
    let mut scalars: Vec<f32> = Vec::with_capacity(npoints);
    match scalar_type.as_str() {
        "short" | "signed_short" => {
            for chunk in payload.chunks_exact(2).take(npoints) { let be = i16::from_be_bytes([chunk[0],chunk[1]]); scalars.push(be as f32); }
            if scalars.iter().all(|v| *v==scalars[0]) { // attempt little-endian retry if all identical
                scalars.clear();
                for chunk in payload.chunks_exact(2).take(npoints) { let le = i16::from_le_bytes([chunk[0],chunk[1]]); scalars.push(le as f32); }
            }
        },
        "unsigned_short" | "ushort" => {
            for chunk in payload.chunks_exact(2).take(npoints) { let be = u16::from_be_bytes([chunk[0],chunk[1]]); scalars.push(be as f32); }
        },
        "float" => { for chunk in payload.chunks_exact(4).take(npoints) { let be = f32::from_be_bytes([chunk[0],chunk[1],chunk[2],chunk[3]]); scalars.push(be); } },
        "double" => { for chunk in payload.chunks_exact(8).take(npoints) { let be = f64::from_be_bytes([chunk[0],chunk[1],chunk[2],chunk[3],chunk[4],chunk[5],chunk[6],chunk[7]]); scalars.push(be as f32); } },
        "int" => { for chunk in payload.chunks_exact(4).take(npoints) { let be = i32::from_be_bytes([chunk[0],chunk[1],chunk[2],chunk[3]]); scalars.push(be as f32); } },
        "unsigned_int" | "uint" => { for chunk in payload.chunks_exact(4).take(npoints) { let be = u32::from_be_bytes([chunk[0],chunk[1],chunk[2],chunk[3]]); scalars.push(be as f32); } },
        "long" => { for chunk in payload.chunks_exact(8).take(npoints) { let be = i64::from_be_bytes([chunk[0],chunk[1],chunk[2],chunk[3],chunk[4],chunk[5],chunk[6],chunk[7]]); scalars.push((be as f64) as f32); } },
        "unsigned_long" | "ulong" => { for chunk in payload.chunks_exact(8).take(npoints) { let be = u64::from_be_bytes([chunk[0],chunk[1],chunk[2],chunk[3],chunk[4],chunk[5],chunk[6],chunk[7]]); scalars.push((be as f64) as f32); } },
        "unsigned_char" | "uchar" => { for &b in payload.iter().take(npoints) { scalars.push(b as f32); } },
        "char" | "signed_char" => { for &b in payload.iter().take(npoints) { scalars.push(b as i8 as f32); } },
        other => return Err(anyhow!(format!("Unsupported binary VTK scalar type '{other}'")))
    }
    if scalars.len() != npoints { return Err(anyhow!(format!("Parsed {} scalars, expected {}", scalars.len(), npoints))); }
    let nx=dims[0]; let ny=dims[1]; let nz=dims[2];
    let x: Vec<f64> = (0..nx).map(|i| origin[0] + i as f64 * spacing[0]).collect();
    let y: Vec<f64> = (0..ny).map(|i| origin[1] + i as f64 * spacing[1]).collect();
    let z: Vec<f64> = (0..nz).map(|i| origin[2] + i as f64 * spacing[2]).collect();
    Volume::new(x,y,z,scalars)
}

fn load_vtk_ascii_rectilinear(full: &str) -> Result<Volume> {
    let mut lines = full.lines().peekable();
    while let Some(l) = lines.peek() { if l.trim_start().starts_with("DATASET") { break; } lines.next(); }
    if let Some(ds) = lines.next() { if !ds.contains("RECTILINEAR_GRID") { return Err(anyhow!("Only RECTILINEAR_GRID supported (ASCII)")); } } else { return Err(anyhow!("Missing DATASET line")); }
    let dims_line = lines.next().ok_or_else(|| anyhow!("Missing DIMENSIONS"))?;
    let dims_tokens: Vec<&str> = dims_line.split_whitespace().collect();
    if dims_tokens.len() < 4 { return Err(anyhow!("Bad DIMENSIONS line")); }
    let nx: usize = dims_tokens[1].parse()?; let ny: usize = dims_tokens[2].parse()?; let nz: usize = dims_tokens[3].parse()?;
    let mut read_coords = |expected: usize, prefix: &str| -> Result<Vec<f64>> {
        let header = lines.next().ok_or_else(|| anyhow!(format!("Missing {prefix} header")))?;
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 3 || !header.starts_with(prefix) { return Err(anyhow!(format!("Bad {prefix} header"))); }
        let _n: usize = parts[1].parse()?;
        let mut vals: Vec<f64> = Vec::new();
        while vals.len() < expected {
            let l = lines.next().ok_or_else(|| anyhow!("Unexpected EOF in coordinates"))?;
            for tok in l.split_whitespace() { vals.push(tok.parse()?); if vals.len()==expected { break; } }
        }
        Ok(vals)
    };
    let x = read_coords(nx, "X_COORDINATES")?;
    let y = read_coords(ny, "Y_COORDINATES")?;
    let z = read_coords(nz, "Z_COORDINATES")?;
    if let Some(peek) = lines.peek() && peek.trim_start().starts_with("CELL_DATA") { lines.next(); }
    for l in lines.by_ref() { if l.trim_start().starts_with("POINT_DATA") { break; } }
    while let Some(l) = lines.peek() { if l.trim().is_empty() { lines.next(); } else { break; } }
    let scalars_header = lines.next().ok_or_else(|| anyhow!("Missing SCALARS header"))?; if !scalars_header.starts_with("SCALARS") { return Err(anyhow!("Expected SCALARS header")); }
    for l in lines.by_ref() { if l.trim_start().starts_with("LOOKUP_TABLE") { break; } }
    let expected_points = nx*ny*nz;
    let mut scalars: Vec<f32> = Vec::with_capacity(expected_points);
    while scalars.len() < expected_points { if let Some(l) = lines.next() { for tok in l.split_whitespace() { scalars.push(tok.parse::<f32>()?); if scalars.len()==expected_points { break; } } } else { break; } }
    if scalars.len() != expected_points { return Err(anyhow!("Incomplete scalar data")); }
    Volume::new(x,y,z,scalars)
}

fn load_vtk_ascii_structured_points(full: &str) -> Result<Volume> {
    let upper = full.to_ascii_uppercase();
    if !upper.contains("STRUCTURED_POINTS") { return Err(anyhow!("Not STRUCTURED_POINTS")); }
    let mut dims=[0usize;3];
    if let Some(line)= full.lines().find(|l| l.to_ascii_uppercase().starts_with("DIMENSIONS")) { let t: Vec<&str>= line.split_whitespace().collect(); if t.len()>=4 { dims[0]=t[1].parse()?; dims[1]=t[2].parse()?; dims[2]=t[3].parse()?; } }
    if dims.contains(&0) { return Err(anyhow!("Missing DIMENSIONS in STRUCTURED_POINTS")); }
    let mut origin=[0f64;3]; if let Some(line)= full.lines().find(|l| l.to_ascii_uppercase().starts_with("ORIGIN")) { let t: Vec<&str>= line.split_whitespace().collect(); if t.len()>=4 { origin[0]=t[1].parse()?; origin[1]=t[2].parse()?; origin[2]=t[3].parse()?; } }
    let mut spacing=[1f64;3]; if let Some(line)= full.lines().find(|l| l.to_ascii_uppercase().starts_with("SPACING")) { let t: Vec<&str>= line.split_whitespace().collect(); if t.len()>=4 { spacing[0]=t[1].parse()?; spacing[1]=t[2].parse()?; spacing[2]=t[3].parse()?; } }
    let expected = dims[0]*dims[1]*dims[2];
    // Locate LOOKUP_TABLE then subsequent numeric lines
    let mut after_lookup = false; let mut scalars: Vec<f32> = Vec::with_capacity(expected);
    for l in full.lines() {
        if !after_lookup { if l.to_ascii_uppercase().starts_with("LOOKUP_TABLE") { after_lookup = true; } continue; }
        for tok in l.split_whitespace() { if let Ok(v) = tok.parse::<f32>() { scalars.push(v); if scalars.len()==expected { break; } } }
        if scalars.len()==expected { break; }
    }
    if scalars.len()!=expected { return Err(anyhow!("Incomplete scalar data in ASCII STRUCTURED_POINTS")); }
    let x: Vec<f64> = (0..dims[0]).map(|i| origin[0] + i as f64 * spacing[0]).collect();
    let y: Vec<f64> = (0..dims[1]).map(|i| origin[1] + i as f64 * spacing[1]).collect();
    let z: Vec<f64> = (0..dims[2]).map(|i| origin[2] + i as f64 * spacing[2]).collect();
    Volume::new(x,y,z,scalars)
}

