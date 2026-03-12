use std::collections::HashMap;

use crate::parser::{Component, PropValue};

struct ComponentDefaults {
    width: Option<f64>,
    height: Option<f64>,
    color: &'static str,
}

fn get_defaults(comp_type: &str) -> ComponentDefaults {
    match comp_type {
        "TForm" => ComponentDefaults { width: Some(640.0), height: Some(480.0), color: "clBtnFace" },
        "TPanel" => ComponentDefaults { width: Some(100.0), height: Some(41.0), color: "clBtnFace" },
        "TGroupBox" => ComponentDefaults { width: Some(185.0), height: Some(105.0), color: "clBtnFace" },
        "TLabel" => ComponentDefaults { width: None, height: None, color: "clBtnFace" },
        "TEdit" => ComponentDefaults { width: Some(121.0), height: Some(21.0), color: "clWindow" },
        "TMemo" => ComponentDefaults { width: Some(185.0), height: Some(89.0), color: "clWindow" },
        "TButton" => ComponentDefaults { width: Some(75.0), height: Some(25.0), color: "clBtnFace" },
        "TBitBtn" => ComponentDefaults { width: Some(75.0), height: Some(25.0), color: "clBtnFace" },
        "TCheckBox" => ComponentDefaults { width: Some(97.0), height: Some(17.0), color: "clBtnFace" },
        "TRadioButton" => ComponentDefaults { width: Some(113.0), height: Some(17.0), color: "clBtnFace" },
        "TComboBox" => ComponentDefaults { width: Some(145.0), height: Some(21.0), color: "clWindow" },
        "TListBox" => ComponentDefaults { width: Some(121.0), height: Some(97.0), color: "clWindow" },
        "TStringGrid" => ComponentDefaults { width: Some(185.0), height: Some(105.0), color: "clWindow" },
        "TPageControl" => ComponentDefaults { width: Some(289.0), height: Some(193.0), color: "clBtnFace" },
        "TTabSheet" => ComponentDefaults { width: None, height: None, color: "clBtnFace" },
        "TImage" => ComponentDefaults { width: Some(105.0), height: Some(105.0), color: "clBtnFace" },
        _ => ComponentDefaults { width: None, height: None, color: "clBtnFace" },
    }
}

fn set_default_number(props: &mut HashMap<String, PropValue>, key: &str, val: f64) {
    props.entry(key.to_string())
        .or_insert(PropValue::Number(val));
}

fn set_default_str(props: &mut HashMap<String, PropValue>, key: &str, val: &str) {
    props.entry(key.to_string())
        .or_insert(PropValue::Str(val.to_string()));
}

fn set_default_bool(props: &mut HashMap<String, PropValue>, key: &str, val: bool) {
    props.entry(key.to_string())
        .or_insert(PropValue::Bool(val));
}

fn set_default_list(props: &mut HashMap<String, PropValue>, key: &str, val: Vec<&str>) {
    props.entry(key.to_string())
        .or_insert(PropValue::List(val.iter().map(|s| s.to_string()).collect()));
}

fn get_number(props: &HashMap<String, PropValue>, key: &str) -> f64 {
    match props.get(key) {
        Some(PropValue::Number(n)) => *n,
        _ => 0.0,
    }
}

fn get_anchors(props: &HashMap<String, PropValue>) -> Vec<String> {
    match props.get("Anchors") {
        Some(PropValue::List(items)) => items.clone(),
        _ => vec!["akLeft".to_string(), "akTop".to_string()],
    }
}

/// Normalize a component's properties with defaults and compute anchors.
pub fn normalize_props(
    comp: &mut Component,
    parent_width: f64,
    parent_height: f64,
) {
    let defaults = get_defaults(&comp.comp_type);

    // Basic defaults
    set_default_number(&mut comp.props, "Left", 0.0);
    set_default_number(&mut comp.props, "Top", 0.0);
    if let Some(w) = defaults.width {
        set_default_number(&mut comp.props, "Width", w);
    }
    if let Some(h) = defaults.height {
        set_default_number(&mut comp.props, "Height", h);
    }
    set_default_str(&mut comp.props, "Color", defaults.color);
    set_default_str(&mut comp.props, "Align", "alNone");
    set_default_list(&mut comp.props, "Anchors", vec!["akLeft", "akTop"]);
    set_default_bool(&mut comp.props, "Visible", true);
    set_default_bool(&mut comp.props, "Enabled", true);
    set_default_number(&mut comp.props, "Font.Size", 8.0);
    set_default_list(&mut comp.props, "Font.Style", vec![]);

    // Compute anchor_right and anchor_bottom
    let anchors = get_anchors(&comp.props);
    let left = get_number(&comp.props, "Left");
    let top = get_number(&comp.props, "Top");
    let width = get_number(&comp.props, "Width");
    let height = get_number(&comp.props, "Height");

    if anchors.iter().any(|a| a == "akRight") {
        let anchor_right = parent_width - (left + width);
        comp.props.insert(
            "anchor_right".to_string(),
            PropValue::Number(anchor_right),
        );
    }

    if anchors.iter().any(|a| a == "akBottom") {
        let anchor_bottom = parent_height - (top + height);
        comp.props.insert(
            "anchor_bottom".to_string(),
            PropValue::Number(anchor_bottom),
        );
    }

    // Get this component's dimensions for children
    let my_width = get_number(&comp.props, "Width");
    let my_height = get_number(&comp.props, "Height");

    // Recursively normalize children
    let mut children = std::mem::take(&mut comp.children);
    for child in children.iter_mut() {
        normalize_props(child, my_width, my_height);
    }
    comp.children = children;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dfm;

    #[test]
    fn test_defaults_applied() {
        let input = r#"object frmMain: TForm
  Width = 800
  Height = 600
  object btn1: TButton
    Left = 10
    Top = 10
  end
end"#;
        let mut comp = parse_dfm(input).unwrap();
        normalize_props(&mut comp, 0.0, 0.0);

        let btn = &comp.children[0];
        assert!(matches!(btn.props.get("Width"), Some(PropValue::Number(n)) if *n == 75.0));
        assert!(matches!(btn.props.get("Height"), Some(PropValue::Number(n)) if *n == 25.0));
        assert!(matches!(btn.props.get("Color"), Some(PropValue::Str(s)) if s == "clBtnFace"));
    }

    #[test]
    fn test_anchor_right() {
        let input = r#"object frmMain: TForm
  Width = 800
  Height = 600
  object edt1: TEdit
    Left = 10
    Top = 10
    Width = 780
    Anchors = [akLeft, akTop, akRight]
  end
end"#;
        let mut comp = parse_dfm(input).unwrap();
        normalize_props(&mut comp, 0.0, 0.0);

        let edt = &comp.children[0];
        // anchor_right = 800 - (10 + 780) = 10
        assert!(matches!(edt.props.get("anchor_right"), Some(PropValue::Number(n)) if *n == 10.0));
    }
}
