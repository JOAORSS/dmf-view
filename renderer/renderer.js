/* renderer.js — DFM Preview renderer (Steps 3–6) */
/* Works in both browser and VSCode WebviewPanel */

// --- Delphi color mapping ---

var DELPHI_COLORS = {
  clBtnFace:   '#f0f0f0',
  clWindow:    '#ffffff',
  clBtnText:   '#000000',
  clHighlight: '#0078d7',
  clNavy:      '#000080',
  clBlue:      '#0000ff',
  clRed:       '#ff0000',
  clGreen:     '#008000',
  clYellow:    '#ffff00',
  clGray:      '#808080',
  clSilver:    '#c0c0c0',
  clBlack:     '#000000',
  clWhite:     '#ffffff',
};

function resolveColor(value) {
  if (!value) return null;
  if (typeof value !== 'string') return null;
  if (value.startsWith('cl')) return DELPHI_COLORS[value] || '#f0f0f0';
  if (value.startsWith('$')) {
    var hex = value.slice(1).padStart(6, '0');
    return '#' + hex.slice(4, 6) + hex.slice(2, 4) + hex.slice(0, 2);
  }
  return value;
}

// --- Escape HTML ---

function escapeHtml(str) {
  if (!str) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

// --- Align CSS mapping (Step 5) ---

var ALIGN_CSS = {
  alNone:   'position: absolute',
  alTop:    'width: 100%; flex-shrink: 0',
  alBottom: 'width: 100%; flex-shrink: 0; margin-top: auto',
  alLeft:   'height: 100%; flex-shrink: 0',
  alRight:  'height: 100%; flex-shrink: 0; margin-left: auto',
  alClient: 'flex: 1; min-width: 0; min-height: 0',
};

// --- Anchors position builder (Step 6) ---

function buildPositionStyle(props) {
  var anchors = props.Anchors || ['akLeft', 'akTop'];
  var hasAkLeft   = anchors.indexOf('akLeft') >= 0;
  var hasAkRight  = anchors.indexOf('akRight') >= 0;
  var hasAkTop    = anchors.indexOf('akTop') >= 0;
  var hasAkBottom = anchors.indexOf('akBottom') >= 0;

  var parts = ['position: absolute'];

  if (hasAkLeft)   parts.push('left: ' + (props.Left || 0) + 'px');
  if (hasAkRight && props.anchor_right != null)
    parts.push('right: ' + props.anchor_right + 'px');
  if (hasAkTop)    parts.push('top: ' + (props.Top || 0) + 'px');
  if (hasAkBottom && props.anchor_bottom != null)
    parts.push('bottom: ' + props.anchor_bottom + 'px');

  if (hasAkLeft && hasAkRight) {
    /* width stretches via left+right */
  } else if (props.Width != null) {
    parts.push('width: ' + props.Width + 'px');
  }

  if (hasAkTop && hasAkBottom) {
    /* height stretches via top+bottom */
  } else if (props.Height != null) {
    parts.push('height: ' + props.Height + 'px');
  }

  return parts.join('; ');
}

// --- Layout helpers (Step 5) ---

function hasAlignedChildren(children) {
  if (!children) return false;
  for (var i = 0; i < children.length; i++) {
    var align = (children[i].props && children[i].props.Align) || 'alNone';
    if (align !== 'alNone') return true;
  }
  return false;
}

function detectFlexDirection(children) {
  var hasTopBottom = false;
  var hasLeftRight = false;
  for (var i = 0; i < children.length; i++) {
    var align = (children[i].props && children[i].props.Align) || 'alNone';
    if (align === 'alTop' || align === 'alBottom') hasTopBottom = true;
    if (align === 'alLeft' || align === 'alRight') hasLeftRight = true;
  }
  if (hasTopBottom && !hasLeftRight) return 'column';
  return 'column'; // default to column for mixed
}

function buildContainerStyle(comp, isRoot) {
  var props = comp.props || {};
  var bg = resolveColor(props.Color) || '#f0f0f0';
  var useFlex = hasAlignedChildren(comp.children);

  var parts = [];
  if (isRoot) {
    parts.push('position: relative');
    parts.push('width: ' + (props.Width || 640) + 'px');
    parts.push('height: ' + (props.Height || 480) + 'px');
  }
  if (useFlex) {
    parts.push('display: flex');
    parts.push('flex-direction: ' + detectFlexDirection(comp.children));
    parts.push('position: relative');
  }
  parts.push('background: ' + bg);
  return parts.join('; ');
}

function buildChildStyle(comp) {
  var props = comp.props || {};
  var align = props.Align || 'alNone';
  var bg = resolveColor(props.Color);

  if (align !== 'alNone') {
    var parts = [ALIGN_CSS[align] || ''];
    if (props.Height != null && (align === 'alTop' || align === 'alBottom')) {
      parts.push('height: ' + props.Height + 'px');
    }
    if (props.Width != null && (align === 'alLeft' || align === 'alRight')) {
      parts.push('width: ' + props.Width + 'px');
    }
    if (bg) parts.push('background: ' + bg);
    return parts.join('; ');
  }

  // alNone — use anchors-based positioning
  var posStyle = buildPositionStyle(props);
  if (bg) posStyle += '; background: ' + bg;
  return posStyle;
}

// --- Component renderers (Step 4) ---

function renderChildren(children) {
  if (!children || children.length === 0) return '';
  var html = '';
  for (var i = 0; i < children.length; i++) {
    html += renderComponent(children[i]);
  }
  return html;
}

function renderTForm(comp) {
  var style = buildContainerStyle(comp, true);
  style += '; border: 2px solid #999; overflow: hidden; box-sizing: border-box';
  return '<div class="dfm-component dfm-form" style="' + style + '">'
    + renderChildren(comp.children) + '</div>';
}

function renderTPanel(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  if (hasAlignedChildren(comp.children)) {
    style += '; display: flex; flex-direction: '
      + detectFlexDirection(comp.children) + '; position: relative';
  }
  style += '; border: 2px solid #d0d0d0; overflow: hidden; box-sizing: border-box';
  var caption = props.Caption ? '<span class="dfm-panel-caption">'
    + escapeHtml(props.Caption) + '</span>' : '';
  return '<div class="dfm-component dfm-panel" style="' + style + '">'
    + caption + renderChildren(comp.children) + '</div>';
}

function renderTGroupBox(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  style += '; border: 1px solid #999; box-sizing: border-box; position: relative';
  var legend = props.Caption
    ? '<legend>' + escapeHtml(props.Caption) + '</legend>' : '';
  return '<fieldset class="dfm-component dfm-groupbox" style="' + style + '">'
    + legend + renderChildren(comp.children) + '</fieldset>';
}

function renderTLabel(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  style += '; font-size: 8pt; color: #000';
  return '<label class="dfm-component dfm-label" style="' + style + '">'
    + escapeHtml(props.Caption || props.Text || '') + '</label>';
}

function renderTEdit(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  var bg = resolveColor(props.Color) || '#fff';
  style += '; border: 1px inset; background: ' + bg + '; box-sizing: border-box';
  var val = props.Text || '';
  return '<input type="text" readonly class="dfm-component dfm-edit" style="'
    + style + '" value="' + escapeHtml(val) + '">';
}

function renderTMemo(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  var bg = resolveColor(props.Color) || '#fff';
  style += '; border: 1px inset; background: ' + bg + '; box-sizing: border-box; resize: none';
  return '<textarea readonly class="dfm-component dfm-memo" style="'
    + style + '">' + escapeHtml(props.Text || '') + '</textarea>';
}

function renderTButton(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  style += '; background: #f0f0f0; border: 2px outset #ccc; box-sizing: border-box; cursor: default';
  return '<button disabled class="dfm-component dfm-button" style="' + style + '">'
    + escapeHtml(props.Caption || '') + '</button>';
}

function renderTBitBtn(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  style += '; background: #f0f0f0; border: 2px outset #ccc; box-sizing: border-box; cursor: default';
  return '<button disabled class="dfm-component dfm-bitbtn" style="' + style + '">'
    + '<span style="display:inline-block;width:16px;height:16px;background:#ccc;'
    + 'vertical-align:middle;margin-right:4px"></span>'
    + escapeHtml(props.Caption || '') + '</button>';
}

function renderTCheckBox(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  return '<label class="dfm-component dfm-checkbox" style="' + style + '">'
    + '<input type="checkbox" disabled> '
    + escapeHtml(props.Caption || '') + '</label>';
}

function renderTRadioButton(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  return '<label class="dfm-component dfm-radiobutton" style="' + style + '">'
    + '<input type="radio" disabled> '
    + escapeHtml(props.Caption || '') + '</label>';
}

function renderTComboBox(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  var bg = resolveColor(props.Color) || '#fff';
  style += '; background: ' + bg + '; border: 1px solid #999; box-sizing: border-box';
  return '<select disabled class="dfm-component dfm-combobox" style="' + style + '">'
    + '<option>' + escapeHtml(props.Text || '') + '</option></select>';
}

function renderTListBox(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  var bg = resolveColor(props.Color) || '#fff';
  style += '; background: ' + bg + '; border: 1px inset; box-sizing: border-box';
  return '<select multiple disabled class="dfm-component dfm-listbox" style="'
    + style + '"></select>';
}

function renderTStringGrid(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  style += '; border-collapse: collapse; box-sizing: border-box; overflow: auto';
  var rows = '';
  for (var r = 0; r < 4; r++) {
    rows += '<tr>';
    for (var c = 0; c < 4; c++) {
      rows += '<td style="border: 1px solid #ccc; padding: 2px 4px; min-width: 40px">&nbsp;</td>';
    }
    rows += '</tr>';
  }
  return '<div class="dfm-component dfm-stringgrid-wrapper" style="' + style + '">'
    + '<table class="dfm-stringgrid" style="border-collapse: collapse; width: 100%; height: 100%">'
    + rows + '</table></div>';
}

function renderTPageControl(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  style += '; box-sizing: border-box; display: flex; flex-direction: column';

  var tabs = '<div class="dfm-tabs" style="display: flex; flex-shrink: 0; border-bottom: 1px solid #999">';
  var children = comp.children || [];
  for (var i = 0; i < children.length; i++) {
    var childCaption = (children[i].props && children[i].props.Caption) || 'Tab ' + i;
    var active = i === 0 ? ' background: #fff; border-bottom: 1px solid #fff; margin-bottom: -1px;' : ' background: #e0e0e0;';
    tabs += '<div class="dfm-tab" style="padding: 4px 12px; border: 1px solid #999; border-bottom: none; cursor: default;'
      + active + '">' + escapeHtml(childCaption) + '</div>';
  }
  tabs += '</div>';

  var pages = '';
  for (var j = 0; j < children.length; j++) {
    var display = j === 0 ? 'display: block' : 'display: none';
    pages += '<div class="dfm-tabsheet" style="flex: 1; position: relative; '
      + display + '; border: 1px solid #999; border-top: none; overflow: hidden">'
      + renderChildren(children[j].children) + '</div>';
  }

  return '<div class="dfm-component dfm-pagecontrol" style="' + style + '">'
    + tabs + pages + '</div>';
}

function renderTTabSheet(comp) {
  var props = comp.props || {};
  return '<div class="dfm-component dfm-tabsheet" style="position: relative; width: 100%; height: 100%">'
    + renderChildren(comp.children) + '</div>';
}

function renderTImage(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  style += '; border: 1px dashed #999; background: #e8e8e8; box-sizing: border-box';
  style += '; display: flex; align-items: center; justify-content: center; color: #999; font-size: 10px';
  return '<div class="dfm-component dfm-image-placeholder" style="' + style + '">Image</div>';
}

function renderUnknown(comp) {
  var props = comp.props || {};
  var style = buildChildStyle(comp);
  style += '; border: 1px dashed #c00; background: #fff0f0; box-sizing: border-box';
  style += '; font-size: 10px; color: #c00; padding: 2px';
  return '<div class="dfm-component dfm-unknown" style="' + style + '">'
    + escapeHtml(comp.type + ': ' + comp.name)
    + renderChildren(comp.children) + '</div>';
}

// --- Main dispatcher ---

var RENDERERS = {
  TForm:         renderTForm,
  TPanel:        renderTPanel,
  TGroupBox:     renderTGroupBox,
  TLabel:        renderTLabel,
  TEdit:         renderTEdit,
  TMemo:         renderTMemo,
  TButton:       renderTButton,
  TBitBtn:       renderTBitBtn,
  TCheckBox:     renderTCheckBox,
  TRadioButton:  renderTRadioButton,
  TComboBox:     renderTComboBox,
  TListBox:      renderTListBox,
  TStringGrid:   renderTStringGrid,
  TPageControl:  renderTPageControl,
  TTabSheet:     renderTTabSheet,
  TImage:        renderTImage,
};

function renderComponent(comp) {
  if (!comp || !comp.type) return '';
  var renderer = RENDERERS[comp.type];
  if (renderer) return renderer(comp);
  return renderUnknown(comp);
}

function renderDFM(json) {
  if (!json) return '<p>No data</p>';
  return renderComponent(json);
}
