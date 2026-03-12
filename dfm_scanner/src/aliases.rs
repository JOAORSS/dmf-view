use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

/// A custom type mapped to its VCL base
#[derive(Debug, serde::Serialize)]
pub struct AliasEntry {
    pub custom_type: String,
    pub vcl_base: String,
    pub depth: u32,
    pub source_file: String,
}

/// VCL base types that terminate inheritance resolution
const VCL_BASE_TYPES: &[&str] = &[
    "TForm",
    "TPanel",
    "TGroupBox",
    "TLabel",
    "TEdit",
    "TMemo",
    "TButton",
    "TBitBtn",
    "TCheckBox",
    "TRadioButton",
    "TComboBox",
    "TListBox",
    "TStringGrid",
    "TPageControl",
    "TTabSheet",
    "TImage",
    "TScrollBox",
    "TSplitter",
    "TStatusBar",
    "TToolBar",
    "TMainMenu",
    "TPopupMenu",
    "TTimer",
    "TDataSource",
    "TDBGrid",
    "TDBEdit",
    "TDBMemo",
    "TDBComboBox",
    "TDBCheckBox",
    "TDBLookupComboBox",
    "TComponent",
    "TControl",
    "TWinControl",
    "TCustomControl",
    // Dialogs
    "TOpenDialog",
    "TSaveDialog",
    "TOpenPictureDialog",
    "TColorDialog",
    "TFontDialog",
    "TPrintDialog",
    // FireDAC
    "TFDConnection",
    "TFDQuery",
    "TFDTable",
    "TFDStoredProc",
];

/// Returns the path to %APPDATA%\CodeInsight\
pub fn get_codeinsight_dir() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let dir = PathBuf::from(appdata).join("CodeInsight");
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

/// Reads PublicoV11 path from config.ini
/// File: %APPDATA%\CodeInsight\config.ini
/// Section: [Paths], key: PublicoV11
pub fn read_public_dir_from_config() -> Option<String> {
    let ci_dir = get_codeinsight_dir()?;
    let config_path = ci_dir.join("config.ini");
    let content = std::fs::read_to_string(config_path).ok()?;
    parse_ini_value(&content, "Paths", "PublicoV11")
}

/// Simple INI parser: finds a value for a given section and key
fn parse_ini_value(content: &str, section: &str, key: &str) -> Option<String> {
    let section_header = format!("[{}]", section);
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed.eq_ignore_ascii_case(&section_header);
            continue;
        }
        if in_section {
            if let Some(idx) = trimmed.find('=') {
                let k = trimmed[..idx].trim();
                if k.eq_ignore_ascii_case(key) {
                    return Some(trimmed[idx + 1..].trim().to_string());
                }
            }
        }
    }
    None
}

/// Returns path to publico.db
pub fn get_public_db_path() -> Option<PathBuf> {
    let ci_dir = get_codeinsight_dir()?;
    let db = ci_dir.join("publico.db");
    if db.is_file() {
        Some(db)
    } else {
        None
    }
}

/// Returns path to local.db given a dfm file path.
/// The project name is detected from the directory containing the .dfm file.
pub fn get_local_db_path(dfm_path: &Path) -> Option<PathBuf> {
    let ci_dir = get_codeinsight_dir()?;
    let project_dir = dfm_path.parent()?;
    let project_name = project_dir.file_name()?.to_str()?;
    let db = ci_dir.join(project_name).join("local.db");
    if db.is_file() {
        Some(db)
    } else {
        None
    }
}

/// Loads all types from a SQLite database and returns a map of lowercase_name→(original_name, parent_lower, source_file).
/// Query: SELECT name, parent_type, file_path FROM symbols WHERE kind=1 AND parent_type != ''
pub fn load_type_parents(
    db_path: &Path,
) -> Result<HashMap<String, (String, String, String)>, String> {
    let conn =
        Connection::open(db_path).map_err(|e| format!("Failed to open db: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT name, parent_type, COALESCE(file_path, '') \
             FROM symbols WHERE kind = 1 AND parent_type IS NOT NULL AND parent_type != ''",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let parent: String = row.get(1)?;
            let file: String = row.get(2)?;
            Ok((name, parent, file))
        })
        .map_err(|e| format!("Failed to query: {}", e))?;

    let mut map = HashMap::new();
    for row in rows {
        if let Ok((name, parent, file)) = row {
            map.insert(
                name.to_lowercase(),
                (name, parent.to_lowercase(), file),
            );
        }
    }
    Ok(map)
}

/// Checks if a type name (lowercase) is a known VCL base type
fn is_vcl_base(name: &str) -> bool {
    VCL_BASE_TYPES
        .iter()
        .any(|t| t.eq_ignore_ascii_case(name))
}

/// Finds the original-cased VCL base type name for a lowercase match
fn find_vcl_base_name(name_lower: &str) -> String {
    VCL_BASE_TYPES
        .iter()
        .find(|t| t.eq_ignore_ascii_case(name_lower))
        .map(|t| t.to_string())
        .unwrap_or_else(|| name_lower.to_string())
}

/// Resolves the inheritance chain transitively until a VCL base type is found.
/// Returns None if no VCL base is reachable within 10 levels.
pub fn resolve_to_vcl_base(
    type_name: &str,
    parents: &HashMap<String, (String, String, String)>,
) -> Option<AliasEntry> {
    let key = type_name.to_lowercase();

    // Get the original casing and source file from the entry
    let (original_name, _, source_file) = parents.get(&key)?;
    let original = original_name.clone();
    let source = source_file.clone();

    let mut current = key;
    let mut depth: u32 = 0;
    let mut visited = std::collections::HashSet::new();

    loop {
        if depth > 10 {
            return None;
        }

        let (_, parent_lower, _) = parents.get(&current)?;
        depth += 1;

        if is_vcl_base(parent_lower) {
            return Some(AliasEntry {
                custom_type: original,
                vcl_base: find_vcl_base_name(parent_lower),
                depth,
                source_file: source,
            });
        }

        // Cycle detection
        if !visited.insert(parent_lower.clone()) {
            return None;
        }

        current = parent_lower.clone();
    }
}

/// Main entry point: builds the complete alias map.
/// Loads types from publico.db and optionally local.db, then resolves
/// each type to its VCL base.
pub fn build_aliases(dfm_path: &Path) -> Vec<AliasEntry> {
    let mut parents: HashMap<String, (String, String, String)> = HashMap::new();

    // Load from publico.db
    if let Some(db_path) = get_public_db_path() {
        if let Ok(map) = load_type_parents(&db_path) {
            parents.extend(map);
        }
    }

    // Load from local.db
    if let Some(db_path) = get_local_db_path(dfm_path) {
        if let Ok(map) = load_type_parents(&db_path) {
            parents.extend(map);
        }
    }

    let type_names: Vec<String> = parents.keys().cloned().collect();
    let mut entries = Vec::new();

    for name in &type_names {
        // Skip types that are already VCL base types
        if is_vcl_base(name) {
            continue;
        }
        if let Some(entry) = resolve_to_vcl_base(name, &parents) {
            entries.push(entry);
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini_value() {
        let content = "[Paths]\nPublicoV11=C:\\Dev\\PublicoV11\n[Other]\nKey=Value\n";
        assert_eq!(
            parse_ini_value(content, "Paths", "PublicoV11"),
            Some("C:\\Dev\\PublicoV11".to_string())
        );
        assert_eq!(
            parse_ini_value(content, "Other", "Key"),
            Some("Value".to_string())
        );
        assert_eq!(parse_ini_value(content, "Paths", "Missing"), None);
        assert_eq!(parse_ini_value(content, "NoSection", "Key"), None);
    }

    #[test]
    fn test_parse_ini_case_insensitive() {
        let content = "[paths]\npublicov11=C:\\Dev\\PublicoV11\n";
        assert_eq!(
            parse_ini_value(content, "Paths", "PublicoV11"),
            Some("C:\\Dev\\PublicoV11".to_string())
        );
    }

    #[test]
    fn test_is_vcl_base() {
        assert!(is_vcl_base("tpanel"));
        assert!(is_vcl_base("TPanel"));
        assert!(is_vcl_base("TFORM"));
        assert!(!is_vcl_base("TCFPainel"));
    }

    #[test]
    fn test_resolve_to_vcl_base_direct() {
        let mut parents = HashMap::new();
        parents.insert(
            "tcfpainel".to_string(),
            ("TCFPainel".to_string(), "tpanel".to_string(), "cfpainel.pas".to_string()),
        );

        let entry = resolve_to_vcl_base("tcfpainel", &parents).unwrap();
        assert_eq!(entry.custom_type, "TCFPainel");
        assert_eq!(entry.vcl_base, "TPanel");
        assert_eq!(entry.depth, 1);
    }

    #[test]
    fn test_resolve_to_vcl_base_transitive() {
        let mut parents = HashMap::new();
        parents.insert(
            "tcfgrid".to_string(),
            ("TCFGrid".to_string(), "tcfbasegrid".to_string(), "cfgrid.pas".to_string()),
        );
        parents.insert(
            "tcfbasegrid".to_string(),
            ("TCFBaseGrid".to_string(), "tstringgrid".to_string(), "cfbasegrid.pas".to_string()),
        );

        let entry = resolve_to_vcl_base("tcfgrid", &parents).unwrap();
        assert_eq!(entry.custom_type, "TCFGrid");
        assert_eq!(entry.vcl_base, "TStringGrid");
        assert_eq!(entry.depth, 2);
    }

    #[test]
    fn test_resolve_cycle_detection() {
        let mut parents = HashMap::new();
        parents.insert(
            "ta".to_string(),
            ("TA".to_string(), "tb".to_string(), "a.pas".to_string()),
        );
        parents.insert(
            "tb".to_string(),
            ("TB".to_string(), "ta".to_string(), "b.pas".to_string()),
        );

        assert!(resolve_to_vcl_base("ta", &parents).is_none());
    }

    #[test]
    fn test_resolve_no_vcl_base() {
        let mut parents = HashMap::new();
        parents.insert(
            "tcustom".to_string(),
            ("TCustom".to_string(), "tunknownbase".to_string(), "custom.pas".to_string()),
        );

        assert!(resolve_to_vcl_base("tcustom", &parents).is_none());
    }

    #[test]
    fn test_find_vcl_base_name() {
        assert_eq!(find_vcl_base_name("tpanel"), "TPanel");
        assert_eq!(find_vcl_base_name("tstringgrid"), "TStringGrid");
        assert_eq!(find_vcl_base_name("tunknown"), "tunknown");
    }

    #[test]
    fn test_load_type_parents_from_sqlite() {
        // Create an in-memory SQLite database for testing
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE symbols (
                id INTEGER PRIMARY KEY,
                file_path TEXT,
                name TEXT COLLATE NOCASE,
                signature TEXT,
                type_name TEXT COLLATE NOCASE,
                data_type TEXT,
                kind INTEGER,
                line INTEGER,
                visibility INTEGER,
                parent_type TEXT
            );
            INSERT INTO symbols (name, kind, parent_type, file_path)
                VALUES ('TCFPainel', 1, 'TPanel', 'cfpainel.pas');
            INSERT INTO symbols (name, kind, parent_type, file_path)
                VALUES ('TCFEdit', 1, 'TEdit', 'cfedit.pas');
            INSERT INTO symbols (name, kind, parent_type, file_path)
                VALUES ('SomeVar', 0, '', 'other.pas');",
        )
        .unwrap();

        // Save to a temp file for load_type_parents
        let tmp_dir = std::env::temp_dir();
        let db_path = tmp_dir.join("test_aliases.db");
        let _ = std::fs::remove_file(&db_path);

        let file_conn = Connection::open(&db_path).unwrap();
        file_conn
            .execute_batch(
                "CREATE TABLE symbols (
                    id INTEGER PRIMARY KEY,
                    file_path TEXT,
                    name TEXT COLLATE NOCASE,
                    signature TEXT,
                    type_name TEXT COLLATE NOCASE,
                    data_type TEXT,
                    kind INTEGER,
                    line INTEGER,
                    visibility INTEGER,
                    parent_type TEXT
                );
                INSERT INTO symbols (name, kind, parent_type, file_path)
                    VALUES ('TCFPainel', 1, 'TPanel', 'cfpainel.pas');
                INSERT INTO symbols (name, kind, parent_type, file_path)
                    VALUES ('TCFEdit', 1, 'TEdit', 'cfedit.pas');
                INSERT INTO symbols (name, kind, parent_type, file_path)
                    VALUES ('SomeVar', 0, '', 'other.pas');",
            )
            .unwrap();
        drop(file_conn);

        let result = load_type_parents(&db_path).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("tcfpainel"));
        assert!(result.contains_key("tcfedit"));
        assert_eq!(result["tcfpainel"].0, "TCFPainel");
        assert_eq!(result["tcfpainel"].1, "tpanel");
        assert_eq!(result["tcfedit"].0, "TCFEdit");
        assert_eq!(result["tcfedit"].1, "tedit");

        let _ = std::fs::remove_file(&db_path);
    }
}
