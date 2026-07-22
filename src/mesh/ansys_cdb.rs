use anyhow::{Result, anyhow};
use regex::Regex;
use std::fs;
use std::collections::HashMap;
use lazy_static::lazy_static;

use super::{Node, Element, ElementKind, Part, Mesh, MeshFormatInfo};

// Pre-compile regex for performance
lazy_static! {
    static ref SCIENTIFIC_NOTATION_RE: Regex = Regex::new(r"-?\d+\.\d+E[+-]\d+").unwrap();
}

pub fn parse_cdb(path: &str, ignore_list: &[String]) -> Result<Mesh> {
    let content = fs::read_to_string(path)?;
    
    // Parse format information first
    let format_info = parse_format_info(&content);
    
    // Parse nodes from NBLOCK
    let nodes = parse_nblock(&content)?;
    
    // Parse elements from EBLOCK  
    let elements = parse_eblock(&content)?;
    
    // Create node index map
    let mut node_index = HashMap::with_capacity(nodes.len());
    for (i, node) in nodes.iter().enumerate() {
        node_index.insert(node.id, i);
    }
    
    // Validate that all elements reference existing nodes
    validate_element_node_references(&elements, &node_index)?;
    
    // For CDB files, we create a single part with all elements
    let part_name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("CDBModel")
        .to_string();
    
    let ignore = ignore_list.iter().any(|s| s == &part_name);
    
    let part = Part {
        name: Some(part_name),
        nodes,
        elements,
        node_index,
        ignore,
    };
    
    Ok(Mesh {
        parts: vec![part],
        mesh_format_info: Some(format_info),
    })
}

fn validate_element_node_references(elements: &[Element], node_index: &HashMap<u32, usize>) -> Result<()> {
    let mut missing_nodes = Vec::new();
    
    // Collect statistics about the nodes and elements
    let node_ids: Vec<u32> = node_index.keys().cloned().collect();
    let min_node_id = node_ids.iter().min().unwrap_or(&0);
    let max_node_id = node_ids.iter().max().unwrap_or(&0);
    
    #[cfg(debug_assertions)]
    println!("DEBUG: Parsed {} nodes with IDs ranging from {} to {}", 
             node_ids.len(), min_node_id, max_node_id);
    
    // Check element node references
    let mut missing_node_set = std::collections::HashSet::new();
    for element in elements {
        for &node_id in &element.nodes {
            if !node_index.contains_key(&node_id) {
                missing_nodes.push((element.id, node_id));
                missing_node_set.insert(node_id);
            }
        }
    }
    
    if !missing_node_set.is_empty() {
        let missing_vec: Vec<u32> = missing_node_set.into_iter().collect();
        let min_missing = missing_vec.iter().min().unwrap_or(&0);
        let max_missing = missing_vec.iter().max().unwrap_or(&0);
        
        println!("DEBUG: {} unique missing nodes with IDs ranging from {} to {}", 
                 missing_vec.len(), min_missing, max_missing);
        
        let sample_errors: Vec<String> = missing_nodes.iter()
            .take(5)  // Show first 5 errors
            .map(|(elem_id, node_id)| format!("Element {} references missing node {}", elem_id, node_id))
            .collect();
        
        return Err(anyhow!(
            "Found {} elements with missing node references. Examples:\n{}{}",
            missing_nodes.len(),
            sample_errors.join("\n"),
            if missing_nodes.len() > 5 { "\n..." } else { "" }
        ));
    }
    
    Ok(())
}

fn parse_nblock(content: &str) -> Result<Vec<Node>> {
    // More flexible regex to handle various spacing
    let nblock_re = Regex::new(r"(?i)NBLOCK,\s*\d+\s*,\s*SOLID\s*,\s*(\d+)\s*,\s*(\d+)")?;
    
    let mut nodes = Vec::new();
    
    if let Some(caps) = nblock_re.find(content) {
        let nblock_start = caps.start();
        
        // Find the start of node data (after the format line)
        let after_nblock = &content[nblock_start..];
        let lines: Vec<&str> = after_nblock.lines().collect();
        
        // Skip NBLOCK line and format line
        let mut data_start = 2;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim().starts_with('(') {
                data_start = i + 1;
                break;
            }
        }
        
        // Reserve space for nodes for better performance
        if let Some(caps) = nblock_re.captures(content)
            && let Ok(node_count) = caps.get(2).unwrap().as_str().parse::<usize>() {
                nodes.reserve(node_count);
            }
        
        // Parse node data until we hit end marker or next section
        let mut parsed_count = 0;
        let mut failed_lines = 0;
        for line in lines.iter().skip(data_start) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("N,") || line.starts_with("EBLOCK") || line == "-1" {
                break;
            }
            
            if let Some(node) = parse_node_line_cdb(line) {
                nodes.push(node);
                parsed_count += 1;
            } else {
                failed_lines += 1;
                if failed_lines <= 5 {  // Show first few failed lines
                    println!("DEBUG: Failed to parse node line {}: '{}'", parsed_count + failed_lines, line);
                }
            }
        }
        
        #[cfg(debug_assertions)]
        println!("DEBUG: Parsed {} nodes, failed to parse {} lines", parsed_count, failed_lines);
    }
    
    if nodes.is_empty() {
        return Err(anyhow!("No nodes found in CDB file"));
    }
    
    Ok(nodes)
}

fn parse_node_line_cdb(line: &str) -> Option<Node> {
    // CDB format: id, 0, 0, x, y, z (with possible scientific notation)
    // Format specification is typically (3i9,6e21.13e3) for NBLOCK
    
    let line = line.trim();
    
    // For CDB format, coordinates can be either concatenated or space-separated
    // Extract the ID
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 3 {
        return None;
    }

    // coordinate extraction is already written so we use that cause that is sensible
    // rather than writing if statements for every pattern
    let id = parts[0].parse::<u32>().ok()?;
    let rest = parts[1..].join(" ");
    let (x, y, z) = extract_three_coordinates(&rest)?;

    Some(Node {id, x, y, z})
}

fn extract_three_coordinates(coords_str: &str) -> Option<(f64, f64, f64)> {
    // Extract exactly 3 scientific notation numbers from a potentially concatenated string
    // Example: "0-4.7796970370000E+001 6.6293449400000E+000-3.1539065550000E+002"
    // Or space-separated: "1.8907520790000E+000 1.7355024250000E-001 -7.9881002630000E+001"
    
    // First try to find scientific notation numbers
    let numbers: Vec<f64> = SCIENTIFIC_NOTATION_RE.find_iter(coords_str)
        .filter_map(|m| parse_scientific_notation(m.as_str()).ok())
        .take(3)
        .collect();
    
    if numbers.len() >= 3 {
        return Some((numbers[0], numbers[1], numbers[2]));
    }
    
    // If regex didn't find 3 numbers, try space-separated parsing
    let parts: Vec<&str> = coords_str.split_whitespace().collect();
    if parts.len() >= 3 && let (Ok(x), Ok(y), Ok(z)) = (
            parse_scientific_notation(parts[0]),
            parse_scientific_notation(parts[1]), 
            parse_scientific_notation(parts[2])
        ) {
            return Some((x, y, z));
        }
    
    None
}

fn parse_eblock(content: &str) -> Result<Vec<Element>> {
    let eblock_re = Regex::new(r"(?i)EBLOCK,\d+,SOLID,\s*(\d+),\s*(\d+)")?;
    
    let mut elements = Vec::new();
    
    if let Some(caps) = eblock_re.find(content) {
        let eblock_start = caps.start();
        
        // Find the start of element data (after the format line)
        let after_eblock = &content[eblock_start..];
        let lines: Vec<&str> = after_eblock.lines().collect();
        
        // Skip EBLOCK line and format line
        let mut data_start = 2;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim().starts_with('(') {
                data_start = i + 1;
                break;
            }
        }
        
        // Parse element data
        let mut i = data_start;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() || line.starts_with("-1") || line.starts_with("MPDATA") {
                break;
            }
            
            if let Some(element) = parse_element_line_cdb(line, &lines, &mut i) {
                elements.push(element);
            }
            i += 1;
        }
    }
    
    if elements.is_empty() {
        return Err(anyhow!("No elements found in CDB file"));
    }
    
    Ok(elements)
}

fn parse_element_line_cdb(line: &str, all_lines: &[&str], current_index: &mut usize) -> Option<Element> {
    // CDB element format: mat_id, elem_type, real, secnum, esys, death, blank, blank, node_count, blank, elem_id, [node_ids...]
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 11 {
        return None;
    }
    
    let element_id: u32 = parts[10].parse().ok()?;
    let node_count: u32 = parts[8].parse().ok()?;
    
    // Determine element type from node count
    let kind = match node_count {
        4 => ElementKind::Tet4,
        10 => ElementKind::Tet10,
        8 => ElementKind::Hex8,
        6 => ElementKind::Wedge6,
        _ => return None, // Unsupported element type
    };
    
    // Collect node IDs from the current line and potentially next lines
    let mut node_ids = Vec::new();
    
    // Get node IDs from current line (starting from position 11)
    for part in parts.iter().skip(11) {
        if let Ok(node_id) = part.parse::<u32>() {
            node_ids.push(node_id);
        }
    }
    
    // If we need more nodes, read from the next line
    if node_ids.len() < node_count as usize && *current_index + 1 < all_lines.len() {
        let next_line = all_lines[*current_index + 1].trim();
        let next_parts: Vec<&str> = next_line.split_whitespace().collect();
        
        for part in next_parts {
            if let Ok(node_id) = part.parse::<u32>() {
                node_ids.push(node_id);
                if node_ids.len() >= node_count as usize {
                    break;
                }
            }
        }
        
        if node_ids.len() >= node_count as usize {
            *current_index += 1; // Skip the next line since we consumed it
        }
    }
    
    if node_ids.len() != node_count as usize {
        return None;
    }
    
    Some(Element {
        id: element_id,
        nodes: node_ids,
        kind,
    })
}

fn parse_scientific_notation(s: &str) -> Result<f64> {
    // Handle ANSYS scientific notation like -3.1376733780000E+001
    // Just normalize E+/E- to e+/e- for Rust parsing
    
    let s = s.trim();
    
    // Replace E+ and E- with e+ and e- for standard Rust parsing
    let normalized = s.replace("E+", "e").replace("E-", "e-");
    
    normalized.parse::<f64>()
        .map_err(|_| anyhow!("Failed to parse number '{}' as scientific notation", s))
}


fn parse_format_info(content: &str) -> MeshFormatInfo {
    let mut format_info = MeshFormatInfo::default();
    
    // Find NBLOCK format specification
    if let Ok(nblock_re) = Regex::new(r"(?i)NBLOCK,.*?\n\s*(\([^)]+\))")
        && let Some(caps) = nblock_re.captures(content) {
            format_info.nblock_format = Some(caps.get(1).unwrap().as_str().to_string());
        }
    
    // Find EBLOCK format specification  
    if let Ok(eblock_re) = Regex::new(r"(?i)EBLOCK,.*?\n\s*(\([^)]+\))")
        && let Some(caps) = eblock_re.captures(content) {
            format_info.eblock_format = Some(caps.get(1).unwrap().as_str().to_string());
        }
    
    format_info
}
