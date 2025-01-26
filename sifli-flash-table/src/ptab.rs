/// ptab.json Parser

use serde::{Deserialize, Serialize};
use anyhow::Result;
use anyhow::anyhow;

const FLASH_CAL_SIZE: u32 = 8*1024;


/// Represents the entire partition table structure
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PartitionTable {
    /// Memory type (e.g., "flash2", "psram1")
    pub(crate) mem: String,
    
    /// Base memory address
    pub(crate) base: String,
    
    /// Regions within this memory segment
    pub(crate) regions: Vec<Region>,
}

/// Represents a specific region within a memory segment
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Region {
    /// Offset from the base address
    pub(crate) offset: String,
    
    /// Maximum size of the region
    pub(crate) max_size: String,
    
    /// Tags describing the region's purpose
    #[serde(default)]
    pub(crate) tags: Option<Vec<String>>,
    
    /// Image name associated with the region
    #[serde(default)]
    pub(crate) img: Option<String>,
    
    /// Executable associated with the region
    #[serde(default)]
    pub(crate) exec: Option<String>,
    
    /// Optional flash table information
    #[serde(default)]
    pub(crate) ftab: Option<FlashTableInfo>,
}

/// Represents additional flash table information
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub(crate) struct FlashTableInfo {
    /// Name of the flash table entry
    pub(crate) name: String,
    
    /// Addressing information
    pub(crate) address: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub start_addr: u32,
    pub size: u32, 
}

pub struct Ptab {
    pub partition_table: Vec<PartitionTable>,
    pub flash_table_info: Info,
    pub hcpu_code_info: Option<Info>,
    pub lcpu_code_info: Option<Info>,
    pub flash_cal_info: Info,
    pub bootloader_patch_ram_info: Info,
    pub bootloader_patch_flash_info: Info,
}

impl Ptab {
    pub fn new(contents: &String) -> Result<Self> {
        let partition_table: Vec<PartitionTable> = serde_json::from_str(contents)?; 
        
        let flash_table_info = find_by_tag(&partition_table, "FLASH_TABLE", "flash")?.unwrap();
        let hcpu_code_info = find_by_tag(&partition_table, "HCPU_FLASH_CODE", "flash")?;
        let lcpu_code_info = find_by_tag(&partition_table, "LCPU_FLASH_CODE", "flash")?;
        let flash_cal_info = Info {
            start_addr: flash_table_info.start_addr + flash_table_info.size,
            size: FLASH_CAL_SIZE,
        };

        let bootloader_patch_ram_info = find_by_tag(&partition_table, "FLASH_BOOT_LOADER", "hpsys_ram")?.unwrap();
        let bootloader_patch_flash_info = find_by_tag(&partition_table, "FLASH_BOOT_LOADER", "flash")?.unwrap();

        Ok(Self {
            partition_table,
            flash_table_info,
            hcpu_code_info,
            lcpu_code_info,
            flash_cal_info,
            bootloader_patch_ram_info,
            bootloader_patch_flash_info,
        })
    }
}

fn find_by_tag(table: &Vec<PartitionTable>, tag: &str, region_contains: &str) -> Result<Option<Info>> {
    // Collect all matching regions across all partition tables
    let matching_regions: Vec<_> = table
        .iter()
        .filter(|pt| pt.mem.contains(region_contains))
        .flat_map(|pt| {
            pt.regions
                .iter()
                .filter(|region| {
                    region.tags
                        .as_ref()
                        .map_or(false, |tags| tags.contains(&tag.to_string()))
                })
                .map(|region| {
                    // Convert hex strings to u32
                    let base = u32::from_str_radix(&pt.base[2..], 16).unwrap_or(0);
                    let offset = u32::from_str_radix(&region.offset[2..], 16).unwrap_or(0);
                    let size = u32::from_str_radix(&region.max_size[2..], 16).unwrap_or(0);
                    
                    Info {
                        start_addr: base + offset,
                        size,
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Handle different search result scenarios
    match matching_regions.len() {
        0 => Ok(None),
        1 => Ok(Some(matching_regions[0].clone())),
        _ => Err(anyhow!("Multiple regions found for tag: {}", tag)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ptab() {
        // Construct the path to the ptab.json file
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test/em-lb525/ptab.json");
        let contents = std::fs::read_to_string(path).unwrap();
        // Parse the partition table
        let result = Ptab::new(&contents);
        
        // Assert that parsing was successful
        assert!(result.is_ok(), "Failed to parse partition table");
        
        let partition_table = result.unwrap();
        
        // Basic validation checks
        assert_eq!(partition_table.partition_table.len(), 4, "Expected 4 memory segments");
        
        // Check first memory segment (flash2)
        let flash2 = &partition_table.partition_table[0];
        assert_eq!(flash2.mem, "flash2");
        assert_eq!(flash2.base, "0x12000000");
        assert_eq!(flash2.regions.len(), 6, "Expected 6 regions in flash2");
        
        // Validate a specific region
        let first_region = &flash2.regions[0];
        assert_eq!(first_region.offset, "0x00000000");
        assert_eq!(first_region.max_size, "0x00008000");
        assert!(first_region.tags.as_ref().unwrap().contains(&"FLASH_TABLE".to_string()));
    }
}