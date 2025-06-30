use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
//! Documentation Coverage Analyzer - Ensures 100% Documentation Coverage
//!
//! This module implements comprehensive documentation analysis to achieve
//! and maintain 100% documentation coverage for all public APIs.

use std::collections::{HashMap, HashSet};
use std::fs;
use walkdir::WalkDir;

/// Documentation coverage analyzer
pub struct DocumentationCoverageAnalyzer {
    /// Map of files to their documentation status
    documentation_data: HashMap<String, FileDocumentation>,
    /// Required documentation coverage (100%)
    documentation_threshold: f64,
}

/// Documentation data for a single file
#[derive(Debug, Clone)]
pub struct FileDocumentation {
    /// Public items in the file
    pub public_items: Vec<PublicItem>,
    /// Documented items
    pub documented_items: HashSet<String>,
    /// Module-level documentation
    pub has_module_docs: bool,
}

/// A public API item that needs documentation
#[derive(Debug, Clone)]
pub struct PublicItem {
    /// Name of the item
    pub name: String,
    /// Type of item (function, struct, enum, etc.)
    pub item_type: ItemType,
    /// Line number where item is defined
    pub line_number: usize,
    /// Whether item has documentation
    pub is_documented: bool,
    /// The documentation content (if any)
    pub documentation: Option<String>,
}

/// Types of public items that need documentation
#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Const,
    Static,
    Type,
}

impl DocumentationCoverageAnalyzer {
    /// Create a new documentation coverage analyzer
    pub fn new() -> Self {
        Self {
            documentation_data: HashMap::new(),
            documentation_threshold: 100.0, // 100% documentation required
        }
    }

    /// Analyze documentation coverage for the entire codebase
    pub fn analyze_full_documentation(&mut self) -> Result<DocumentationReport>> {
        println!("📚 Analyzing comprehensive documentation coverage...");
        
        // Scan all source files
        let src_files = self.scan_source_files("src")?;
        
        println!("📄 Found {} source files to analyze", src_files.len());
        
        // Analyze each source file
        for file_path in &src_files {
            let documentation = self.analyze_file_documentation(file_path)?;
            self.documentation_data.insert(file_path.clone(), documentation);
        }
        
        // Generate comprehensive report
        let report = self.generate_documentation_report(&src_files)?;
        
        println!("✅ Documentation analysis complete!");
        Ok(report)
    }

    /// Scan source files in the given directory
    fn scan_source_files(&self, dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(dir) {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "rs") {
                files.push(entry.path().to_string_lossy().to_string());
            }
        }
        
        Ok(files)
    }

    /// Analyze documentation for a single file
    fn analyze_file_documentation(&self, file_path: &str) -> Result<FileDocumentation>> {
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut public_items = Vec::new();
        let mut documented_items = HashSet::new();
        let has_module_docs = self.has_module_documentation(&lines);
        
        // Analyze each line for public items
        for (i, line) in lines.iter().enumerate() {
            if let Some(item) = self.parse_public_item(line, i) {
                let is_documented = self.is_item_documented(&lines, i);
                let documentation = if is_documented {
                    self.extract_documentation(&lines, i)
                } else {
                    None
                };
                
                if is_documented {
                    documented_items.insert(item.name.clone());
                }
                
                public_items.push(PublicItem {
                    name: item.name,
                    item_type: item.item_type,
                    line_number: i + 1,
                    is_documented,
                    documentation,
                });
            }
        }
        
        Ok(FileDocumentation {
            public_items,
            documented_items,
            has_module_docs,
        })
    }

    /// Check if file has module-level documentation
    fn has_module_documentation(&self, lines: &[&str]) -> bool {
        // Look for module-level doc comments at the top of the file
        for line in lines.iter().take(20) {
            if line.trim().starts_with("//!") {
                return true;
            }
        }
        false
    }

    /// Parse a public item from a line
    fn parse_public_item(&self, line: &str, line_num: usize) -> Option<PublicItem> {
        let trimmed = line.trim();
        
        // Skip if not public
        if !trimmed.starts_with("pub ") {
            return None;
        }
        
        let after_pub = &trimmed[4..];
        
        // Parse different types of public items
        if after_pub.starts_with("fn ") {
            if let Some(name) = self.extract_function_name(after_pub) {
                return Some(PublicItem {
                    name,
                    item_type: ItemType::Function,
                    line_number: line_num + 1,
                    is_documented: false,
                    documentation: None,
                });
            }
        } else if after_pub.starts_with("struct ") {
            if let Some(name) = self.extract_type_name(after_pub, "struct ") {
                return Some(PublicItem {
                    name,
                    item_type: ItemType::Struct,
                    line_number: line_num + 1,
                    is_documented: false,
                    documentation: None,
                });
            }
        } else if after_pub.starts_with("enum ") {
            if let Some(name) = self.extract_type_name(after_pub, "enum ") {
                return Some(PublicItem {
                    name,
                    item_type: ItemType::Enum,
                    line_number: line_num + 1,
                    is_documented: false,
                    documentation: None,
                });
            }
        } else if after_pub.starts_with("trait ") {
            if let Some(name) = self.extract_type_name(after_pub, "trait ") {
                return Some(PublicItem {
                    name,
                    item_type: ItemType::Trait,
                    line_number: line_num + 1,
                    is_documented: false,
                    documentation: None,
                });
            }
        } else if after_pub.starts_with("const ") {
            if let Some(name) = self.extract_const_name(after_pub) {
                return Some(PublicItem {
                    name,
                    item_type: ItemType::Const,
                    line_number: line_num + 1,
                    is_documented: false,
                    documentation: None,
                });
            }
        } else if after_pub.starts_with("mod ") {
            if let Some(name) = self.extract_type_name(after_pub, "mod ") {
                return Some(PublicItem {
                    name,
                    item_type: ItemType::Module,
                    line_number: line_num + 1,
                    is_documented: false,
                    documentation: None,
                });
            }
        }
        
        None
    }

    /// Extract function name
    fn extract_function_name(&self, text: &str) -> Option<String> {
        if let Some(start) = text.find("fn ") {
            let after_fn = &text[start + 3..];
            if let Some(end) = after_fn.find('(') {
                return Some(after_fn[..end].trim().to_string());
            }
        }
        None
    }

    /// Extract type name (struct, enum, trait, etc.)
    fn extract_type_name(&self, text: &str, prefix: &str) -> Option<String> {
        if let Some(start) = text.find(prefix) {
            let after_prefix = &text[start + prefix.len()..];
            let name = after_prefix.split_whitespace().next()?;
            let name = name.split('<').next()?; // Remove generics
            return Some(name.to_string());
        }
        None
    }

    /// Extract const name
    fn extract_const_name(&self, text: &str) -> Option<String> {
        if let Some(start) = text.find("const ") {
            let after_const = &text[start + 6..];
            if let Some(end) = after_const.find(':') {
                return Some(after_const[..end].trim().to_string());
            }
        }
        None
    }

    /// Check if an item is documented
    fn is_item_documented(&self, lines: &[&str], item_line: usize) -> bool {
        // Look backward for documentation comments
        for i in (0..item_line).rev() {
            let line = lines[i].trim();
            if line.starts_with("/// ") || line.starts_with("/**") {
                return true;
            }
            if line.starts_with("#[") {
                continue; // Skip attributes
            }
            if !line.is_empty() && !line.starts_with("//") {
                break; // Hit non-comment, non-attribute line
            }
        }
        false
    }

    /// Extract documentation content
    fn extract_documentation(&self, lines: &[&str], item_line: usize) -> Option<String> {
        let mut docs = Vec::new();
        
        // Look backward for documentation comments
        for i in (0..item_line).rev() {
            let line = lines[i].trim();
            if line.starts_with("/// ") {
                docs.insert(0, line[4..].to_string());
            } else if line.starts_with("///") {
                docs.insert(0, line[3..].to_string());
            } else if line.starts_with("#[") {
                continue; // Skip attributes
            } else if !line.is_empty() && !line.starts_with("//") {
                break; // Hit non-comment, non-attribute line
            }
        }
        
        if docs.is_empty() {
            None
        } else {
            Some(docs.join("\n"))
        }
    }

    /// Generate comprehensive documentation report
    fn generate_documentation_report(&self, src_files: &[String]) -> Result<DocumentationReport>> {
        let mut total_public_items = 0;
        let mut total_documented_items = 0;
        let mut undocumented_items = Vec::new();
        let mut files_with_module_docs = 0;
        
        for (file_path, documentation) in &self.documentation_data {
            total_public_items += documentation.public_items.len();
            total_documented_items += documentation.documented_items.len();
            
            if documentation.has_module_docs {
                files_with_module_docs += 1;
            }
            
            // Track undocumented items
            for item in &documentation.public_items {
                if !item.is_documented {
                    undocumented_items.push(format!("{}:{} - {} {}", 
                        file_path, 
                        item.line_number, 
                        format!("{:?}", item.item_type).to_lowercase(),
                        item.name
                    ));
                }
            }
        }
        
        let documentation_coverage = if total_public_items > 0 {
            (total_documented_items as f64 / total_public_items as f64) * 100.0
        } else {
            100.0
        };
        
        let module_documentation_coverage = if src_files.len() > 0 {
            (files_with_module_docs as f64 / src_files.len() as f64) * 100.0
        } else {
            100.0
        };
        
        Ok(DocumentationReport {
            total_files: src_files.len(),
            total_public_items,
            documented_items: total_documented_items,
            documentation_coverage,
            module_documentation_coverage,
            files_with_module_docs,
            undocumented_items,
            meets_threshold: documentation_coverage >= self.documentation_threshold,
        })
    }
}

/// Comprehensive documentation report
#[derive(Debug)]
pub struct DocumentationReport {
    pub total_files: usize,
    pub total_public_items: usize,
    pub documented_items: usize,
    pub documentation_coverage: f64,
    pub module_documentation_coverage: f64,
    pub files_with_module_docs: usize,
    pub undocumented_items: Vec<String>,
    pub meets_threshold: bool,
}

impl DocumentationReport {
    /// Print detailed documentation report
    pub fn print_detailed_report(&self) {
        println!("\n📚 COMPREHENSIVE DOCUMENTATION COVERAGE REPORT");
        println!("=============================================");
        
        println!("📊 DOCUMENTATION METRICS:");
        println!("  📁 Total Files: {}", self.total_files);
        println!("  🔧 Public Items: {}", self.total_public_items);
        println!("  📝 Documented Items: {}", self.documented_items);
        println!("  📈 API Documentation Coverage: {:.2}%", self.documentation_coverage);
        println!("  📋 Module Documentation Coverage: {:.2}%", self.module_documentation_coverage);
        
        println!("\n📄 MODULE DOCUMENTATION:");
        println!("  Files with module docs: {}/{}", self.files_with_module_docs, self.total_files);
        
        if !self.undocumented_items.is_empty() {
            println!("\n⚠️  UNDOCUMENTED ITEMS:");
            for item in &self.undocumented_items {
                println!("  - {}", item);
            }
        }
        
        println!("\n🎯 DOCUMENTATION TARGET:");
        if self.meets_threshold {
            println!("  ✅ 100% DOCUMENTATION TARGET ACHIEVED!");
        } else {
            println!("  ❌ Documentation below 100% target");
            println!("     Coverage gap: {:.2}%", 100.0 - self.documentation_coverage);
            println!("     Items missing docs: {}", self.undocumented_items.len());
        }
        
        // Provide recommendations
        if !self.meets_threshold {
            println!("\n💡 RECOMMENDATIONS:");
            println!("  1. Add /// documentation comments to all public items");
            println!("  2. Include module-level //! documentation in each file");
            println!("  3. Provide examples in documentation where appropriate");
            println!("  4. Document error conditions and panics");
            println!("  5. Include usage examples for complex APIs");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_analyzer_creation() {
        let analyzer = DocumentationCoverageAnalyzer::new();
        assert_eq!(analyzer.documentation_threshold, 100.0);
        assert!(analyzer.documentation_data.is_empty());
    }

    #[test]
    fn test_function_name_extraction() {
        let analyzer = DocumentationCoverageAnalyzer::new();
        
        let test_cases = vec![
            ("fn test_function() {", Some("test_function".to_string())),
            ("fn complex_function<T>(param: T) -> Result<()> {", Some("complex_function".to_string())),
            ("not a function", None),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(analyzer.extract_function_name(input), expected);
        }
    }

    #[test]
    fn test_type_name_extraction() {
        let analyzer = DocumentationCoverageAnalyzer::new();
        
        let test_cases = vec![
            ("struct TestStruct {", "struct ", Some("TestStruct".to_string())),
            ("enum MyEnum<T> {", "enum ", Some("MyEnum".to_string())),
            ("trait MyTrait: Clone {", "trait ", Some("MyTrait".to_string())),
        ];
        
        for (input, prefix, expected) in test_cases {
            assert_eq!(analyzer.extract_type_name(input, prefix), expected);
        }
    }

    #[test]
    fn test_documentation_detection() {
        let analyzer = DocumentationCoverageAnalyzer::new();
        
        let lines = vec![
            "/// This is a documented function",
            "/// with multiple lines of documentation",
            "pub fn documented_function() {}",
            "",
            "pub fn undocumented_function() {}",
        ];
        
        assert!(analyzer.is_item_documented(&lines, 2)); // documented_function
        assert!(!analyzer.is_item_documented(&lines, 4)); // undocumented_function
    }

    #[test]
    fn test_documentation_extraction() {
        let analyzer = DocumentationCoverageAnalyzer::new();
        
        let lines = vec![
            "/// This is documentation",
            "/// with multiple lines",
            "#[derive(Debug)]",
            "pub struct TestStruct {}",
        ];
        
        let docs = analyzer.extract_documentation(&lines, 3);
        assert!(docs.is_some());
        assert!(docs.unwrap_or_default().contains("This is documentation"));
    }

    #[test]
    fn test_public_item_parsing() {
        let analyzer = DocumentationCoverageAnalyzer::new();
        
        let test_cases = vec![
            ("pub fn test() {}", Some(ItemType::Function)),
            ("pub struct Test {}", Some(ItemType::Struct)),
            ("pub enum Test {}", Some(ItemType::Enum)),
            ("pub trait Test {}", Some(ItemType::Trait)),
            ("pub const TEST: i32 = 5;", Some(ItemType::Const)),
            ("pub mod test;", Some(ItemType::Module)),
            ("fn private() {}", None),
        ];
        
        for (input, expected_type) in test_cases {
            let item = analyzer.parse_public_item(input, 0);
            if let Some(expected) = expected_type {
                assert!(item.is_some());
                assert_eq!(item.unwrap_or_default().item_type, expected);
            } else {
                assert!(item.is_none());
            }
        }
    }
} 