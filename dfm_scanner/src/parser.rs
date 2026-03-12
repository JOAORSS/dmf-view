use serde::Serialize;
use std::collections::HashMap;

/// A parsed DFM value.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum PropValue {
    Number(f64),
    Bool(bool),
    Str(String),
    List(Vec<String>),
}

/// A parsed DFM component node.
#[derive(Debug, Clone, Serialize)]
pub struct Component {
    #[serde(rename = "type")]
    pub comp_type: String,
    pub name: String,
    pub props: HashMap<String, PropValue>,
    pub children: Vec<Component>,
}

/// Parse a DFM file content into a Component tree.
pub fn parse_dfm(input: &str) -> Option<Component> {
    let lines: Vec<&str> = input.lines().collect();
    let mut pos = 0;
    parse_object(&lines, &mut pos)
}

fn parse_object(lines: &[&str], pos: &mut usize) -> Option<Component> {
    while *pos < lines.len() {
        let trimmed = lines[*pos].trim();
        if trimmed.starts_with("object ") || trimmed.starts_with("inherited ") {
            break;
        }
        *pos += 1;
    }
    if *pos >= lines.len() {
        return None;
    }
    let header = lines[*pos].trim();
    *pos += 1;

    let (name, comp_type) = parse_header(header)?;

    let mut props = HashMap::new();
    let mut children = Vec::new();

    while *pos < lines.len() {
        let trimmed = lines[*pos].trim();

        if trimmed == "end" {
            *pos += 1;
            break;
        }

        if trimmed.starts_with("object ") || trimmed.starts_with("inherited ") {
            if let Some(child) = parse_object(lines, pos) {
                children.push(child);
            }
            continue;
        }

        if let Some((key, value)) = parse_property(trimmed) {
            if is_binary_block(&value) {
                skip_binary_block(lines, pos);
            } else if is_list_block(&value) {
                skip_list_block(lines, pos);
            } else {
                props.insert(key, parse_value(&value));
                *pos += 1;
            }
        } else {
            *pos += 1;
        }
    }

    Some(Component {
        comp_type,
        name,
        props,
        children,
    })
}

fn parse_header(header: &str) -> Option<(String, String)> {
    let header = header
        .strip_prefix("object ")
        .or_else(|| header.strip_prefix("inherited "))?;
    let parts: Vec<&str> = header.splitn(2, ':').collect();
    if parts.len() == 2 {
        Some((
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
        ))
    } else {
        None
    }
}

fn parse_property(line: &str) -> Option<(String, String)> {
    let idx = line.find('=')?;
    let key = line[..idx].trim().to_string();
    let value = line[idx + 1..].trim().to_string();
    Some((key, value))
}

fn parse_value(value: &str) -> PropValue {
    // Boolean
    if value.eq_ignore_ascii_case("true") {
        return PropValue::Bool(true);
    }
    if value.eq_ignore_ascii_case("false") {
        return PropValue::Bool(false);
    }

    // Set: [akLeft, akTop]
    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len() - 1];
        if inner.trim().is_empty() {
            return PropValue::List(Vec::new());
        }
        let items: Vec<String> = inner
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        return PropValue::List(items);
    }

    // String: 'text'
    if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
        return PropValue::Str(value[1..value.len() - 1].to_string());
    }

    // Number (integer or float)
    if let Ok(n) = value.parse::<i64>() {
        return PropValue::Number(n as f64);
    }
    if let Ok(n) = value.parse::<f64>() {
        return PropValue::Number(n);
    }

    // Identifier (e.g., alTop, clBtnFace)
    PropValue::Str(value.to_string())
}

fn is_binary_block(value: &str) -> bool {
    value == "{"
}

fn skip_binary_block(lines: &[&str], pos: &mut usize) {
    *pos += 1;
    while *pos < lines.len() {
        if lines[*pos].trim().ends_with('}') {
            *pos += 1;
            return;
        }
        *pos += 1;
    }
}

fn is_list_block(value: &str) -> bool {
    value == "("
}

fn skip_list_block(lines: &[&str], pos: &mut usize) {
    *pos += 1;
    while *pos < lines.len() {
        if lines[*pos].trim() == ")" || lines[*pos].trim().ends_with(')') {
            *pos += 1;
            return;
        }
        *pos += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_form() {
        let input = r#"object frmCadastro: TForm
  Left = 100
  Top = 100
  Width = 800
  Height = 600
  Caption = 'Cadastro de Clientes'
  object pnlTopo: TPanel
    Left = 0
    Top = 0
    Width = 800
    Height = 50
    Align = alTop
    object lblTitulo: TLabel
      Left = 8
      Top = 16
      Caption = 'Clientes'
      Anchors = [akLeft, akTop]
    end
  end
end"#;
        let comp = parse_dfm(input).unwrap();
        assert_eq!(comp.comp_type, "TForm");
        assert_eq!(comp.name, "frmCadastro");
        assert_eq!(comp.children.len(), 1);

        let panel = &comp.children[0];
        assert_eq!(panel.comp_type, "TPanel");
        assert_eq!(panel.children.len(), 1);

        let label = &panel.children[0];
        assert_eq!(label.comp_type, "TLabel");
    }

    #[test]
    fn test_parse_values() {
        assert!(matches!(parse_value("100"), PropValue::Number(n) if n == 100.0));
        assert!(matches!(parse_value("true"), PropValue::Bool(true)));
        assert!(matches!(parse_value("false"), PropValue::Bool(false)));
        assert!(matches!(parse_value("'Hello'"), PropValue::Str(ref s) if s == "Hello"));
        assert!(matches!(parse_value("alTop"), PropValue::Str(ref s) if s == "alTop"));

        if let PropValue::List(items) = parse_value("[akLeft, akTop]") {
            assert_eq!(items, vec!["akLeft", "akTop"]);
        } else {
            panic!("Expected List");
        }
    }
}
