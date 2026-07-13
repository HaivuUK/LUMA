use anyhow::{Result, anyhow};
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::fs::File;

use super::{Node, Element, ElementKind, Part, Mesh, MeshFormatInfo};

pub fn parse_inp(path: &str, ignore_list: &[String]) -> Result<Mesh> {
    // Uses buffererd reading for better I/O performance
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut content = String::new();
    let mut format_info = MeshFormatInfo::default();

    // Read file with pre-allocated buffer
    for line in reader.lines() {
        content.push_str(&line?);
        content.push('\n');
    }

    // Tolerantt part regex that allows CRLF, spaces, and case variants
    // dot matches newline vis (?s)
    let part_re = Regex::new(r"(?is)\*Part\s*,?\s*name=([^\r\n]+)[\r\n]+(.*?)\*End\s+Part")?;
    let mut parts = Vec::new();
    for caps in part_re.captures_iter(&content) {
        let name = caps.get(1).unwrap().as_str().trim().to_string();
        let body = caps.get(2).unwrap().as_str();
        let (nodes, elements) = parse_part_body(body)?;
        let ignore = ignore_list.iter().any(|s | s == &name);
        let mut map = std::collections::HashMap::with_capacity(nodes.len());
        for (i, n) in nodes.iter().enumerate() {
            map.insert(n.id, i);
        }
        parts.push(Part {
            name: Some(name),
            nodes,
            elements,
            node_index: map,
            ignore
        });
    }
    if parts.is_empty() {
        // Some abaqus exports use a weird blocking and are valid as partless INP files this stuff fixes
        // that?
        let (nodes, elements) = parse_part_body(&content)?;
        if nodes.is_empty() || elements.is_empty() {
            return Err(anyhow!("No parts parsed from Abaqus INP"));
        }
        let mut map = std::collections::HashMap::with_capacity(nodes.len());
        for (i, n) in nodes.iter().enumerate() {
            map.insert(n.id, i);
        }
        parts.push(Part {
            name: None, // Should handle safely with Option<String>
            nodes,
            elements,
            node_index: map,
            ignore: false,
        });
    }
    Ok(Mesh {parts, mesh_format_info: None })
}

fn parse_part_body(body: &str) -> Result<(Vec<Node>, Vec<Element>)> {
    // Pre-allocate with estimated capacity
    let mut nodes: Vec<Node> = Vec::with_capacity(body.lines().count() / 4); // rough estimate
    let mut elements: Vec<Element> = Vec::with_capacity(body.lines().count() / 8); // rough estimate

    let lines: Vec<&str> = body.lines().collect();

    // Manual scan for *Node and *Element sections (case-insensitive)
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i].trim();
        if l.to_ascii_lowercase().starts_with("*node") {
            i += 1; // move to next line after *Node
            while i < lines.len() {
                let ln = lines[i].trim();
                if is_abaqus_keyword_line(ln) { break; }
                if ln.starts_with("**") {
                    i += 1;
                    continue;
                }
                if !ln.is_empty() && let Some(n) = parse_node_line(ln) {
                    nodes.push(n);
                }
                i += 1;
            }
        } else if l.to_ascii_lowercase().starts_with("*element") {
            // extract type
            let etype = l.split(',').find_map(|tok| {
                let t = tok.trim();
                if t.to_ascii_lowercase().starts_with("type=") {
                    Some(t.split('=').nth(1).unwrap().trim())
                } else {
                    None
                }
            }).ok_or_else(|| anyhow!("Element type missing"))?;

            let kind = match etype {
                "C3D4" => ElementKind::Tet4,
                "C3D10" => ElementKind::Tet10,
                "C3D8" => ElementKind::Hex8,
                "C3D6" => ElementKind::Wedge6,
                other => return Err(anyhow!(format!("Unsupported element type {other}")))
            };

            i += 1; // move to next line after *Element
            while i < lines.len() {
                let ln = lines[i].trim();
                if is_abaqus_keyword_line(ln) { break; }
                if ln.starts_with("**") {
                    i += 1;
                    continue;
                }
                if !ln.is_empty() && let Some(ele) = parse_element_line(ln, &kind) {
                    elements.push(ele);
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    // Shrink vectors to actual size to save memory
    nodes.shrink_to_fit();
    elements.shrink_to_fit();

    Ok((nodes, elements))
}

#[inline]
fn parse_node_line(line: &str) -> Option<Node> {
    // Avoid allocating Vec by using split directly
    let mut parts = line.split(',');
    let id: u32 = parts.next()?.trim().parse().ok()?;
    let x: f64 = parts.next()?.trim().parse().ok()?;
    let y: f64 = parts.next()?.trim().parse().ok()?;
    let z: f64 = parts.next()?.trim().parse().ok()?;
    Some(Node { id, x, y, z })
}

#[inline]
fn parse_element_line(line: &str, kind: &ElementKind) -> Option<Element> {
    let mut parts = line.split(',');
    let id: u32 = parts.next()?.trim().parse().ok()?;
    let expected = match kind {
        ElementKind::Tet4 => 4,
        ElementKind::Tet10 => 10,
        ElementKind::Hex8 => 8,
        ElementKind::Wedge6 => 6
    };

    // Pre-allocate with exact capacity
    let mut nodes = Vec::with_capacity(expected);
    for part in parts {
        if let Ok(node_id) = part.trim().parse::<u32>() {
            nodes.push(node_id);
        }
    }

    if nodes.len() != expected { return None; }
    Some(Element { id, nodes, kind: kind.clone() })
}

pub fn parse_elset_data_line(data_line: &str, is_generate: bool, ids: &mut Vec<u32>) {
    let nums: Vec<u32> = data_line
        .split(',')
        .filter_map(|tok| tok.trim().parse::<u32>().ok())
        .collect();

    if is_generate {
        for chunk in nums.chunks(3) {
            if chunk.len() >= 2 {
                let start = chunk[0];
                let end = chunk[1];
                let step = if chunk.len() >= 3 { chunk[2].max(1) } else { 1 };
                let mut v = start;
                while v <= end {
                    ids.push(v);
                    match v.checked_add(step) {
                        Some(next) => v = next,
                        None => break,
                    }
                }
            }
        }
    } else {
        ids.extend(nums);
    }
}

pub fn extract_kwarg(keyword_line: &str, key: &str) -> Option<String> {
    let key_lower = key.to_ascii_lowercase();
    keyword_line
        .split(',')
        .find_map(|token| {
            let trimmed = token.trim();
            let mut parts = trimmed.splitn(2, '=');
            let lhs = parts.next()?.trim().to_ascii_lowercase();
            let rhs = parts.next()?.trim();
            if lhs == key_lower {
                Some(rhs.to_string())
            } else {
                None
            }
        })
}

#[inline]
pub fn is_abaqus_keyword_line(line: &str) -> bool {
    line.starts_with('*') && !line.starts_with("**")
}

#[inline]
pub fn parse_element_id_from_line(data_line: &str) -> Option<u32> {
    data_line.split(',').next()?.trim().parse::<u32>().ok()
}

pub fn get_material_modulus_case_insensitive<'a>(
    mat_map: &'a std::collections::HashMap<String, f64>,
    material_name: &str,
) -> Option<&'a f64> {
    if let Some(v) = mat_map.get(material_name) {
        return Some(v);
    }
    let target = material_name.to_ascii_lowercase();
    mat_map.iter().find_map(|(k, v)| {
        if k.to_ascii_lowercase() == target {
            Some(v)
        } else {
            None
        }
    })
}

pub fn get_elset_ids_case_insensitive<'a>(
    elset_members: &'a std::collections::HashMap<String, Vec<u32>>,
    elset_name: &str,
) -> Option<&'a Vec<u32>> {
    if let Some(v) = elset_members.get(elset_name) {
        return Some(v);
    }
    let target = elset_name.to_ascii_lowercase();
    elset_members.iter().find_map(|(k, v)| {
        if k.to_ascii_lowercase() == target {
            Some(v)
        } else {
            None
        }
    })
}
