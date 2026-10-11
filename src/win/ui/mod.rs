mod win32;

use crate::i18n::{self, Lang};
use crate::power::catalog;
use crate::power::catalog_subgroups;
use crate::power::scan::{scan_all, SettingRef, SubgroupNode};
use crate::power::visibility::{
    apply_mode, mode_from_attributes, VisibilityMode,
};
use crate::win::wide::to_wide;
use std::ffi::c_void;
use std::ptr;
use win32::*;

pub const ID_TREE: i32 = 1001;
pub const ID_TITLE: i32 = 1002;
pub const ID_DESC: i32 = 1003;
pub const ID_VIS_LABEL: i32 = 1004;
pub const ID_VIS_MODE: i32 = 1005;
pub const ID_LANG_LABEL: i32 = 1006;
pub const ID_LANG: i32 = 1007;

const WM_TREE_SYNC: UINT = 0x8000 + 42;
const WM_LBUTTONDOWN: UINT = 0x0201;
const WM_LBUTTONUP: UINT = 0x0202;
const WM_KEYUP: UINT = 0x0101;
const WM_KEYDOWN: UINT = 0x0100;
const VK_UP: WPARAM = 0x26;
const VK_DOWN: WPARAM = 0x28;
const VK_LEFT: WPARAM = 0x25;
const VK_RIGHT: WPARAM = 0x27;
const VK_HOME: WPARAM = 0x24;
const VK_END: WPARAM = 0x23;
const VK_PRIOR: WPARAM = 0x21;
const VK_NEXT: WPARAM = 0x22;

fn is_tree_nav_key(wparam: WPARAM) -> bool {
    matches!(
        wparam,
        VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT | VK_HOME | VK_END | VK_PRIOR | VK_NEXT
    )
}

struct AppState {
    hwnd: HWND,
    tree: HWND,
    title: HWND,
    desc: HWND,
    vis_label: HWND,
    vis_mode: HWND,
    lang_label: HWND,
    lang_combo: HWND,
    lang: Lang,
    nodes: Vec<SubgroupNode>,
    settings_flat: Vec<SettingRef>,
    /// HTREEITEM → индекс в `settings_flat` (надёжнее, чем lParam дерева).
    tree_item_index: Vec<(isize, usize)>,
    /// HTREEITEM → индекс в `nodes` (раздел/подгруппа).
    tree_group_index: Vec<(isize, usize)>,
    root_item: isize,
    selected_index: Option<usize>,
    /// Selected subgroup node index when a section (not a leaf) is active.
    selected_group: Option<usize>,
    /// Подавляет запись в реестр при программной смене списка.
    suppress_vis_apply: bool,
    suppress_lang_apply: bool,
    font: HFONT,
}

unsafe fn state_mut(hwnd: HWND) -> Option<&'static mut AppState> {
    let p = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
    if p.is_null() {
        None
    } else {
        Some(&mut *p)
    }
}

fn with_state<F, R>(hwnd: HWND, f: F) -> R
where
    F: FnOnce(&mut AppState) -> R,
{
    unsafe { f(state_mut(hwnd).expect("AppState")) }
}

fn msg_box(hwnd: HWND, text: &str, title: &str, flags: u32) -> i32 {
    let t = to_wide(text);
    let c = to_wide(title);
    unsafe { MessageBoxW(hwnd, t.as_ptr(), c.as_ptr(), flags) }
}

fn set_text(hwnd: HWND, text: &str) {
    let w = to_wide(text);
    unsafe { SetWindowTextW(hwnd, w.as_ptr()) };
}

/// Win32 Rich Edit отображает только `\r\n`, не `\n`.
fn newlines_for_edit(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\n', "\r\n")
}

fn utf16_line_spans(w: &[u16]) -> Vec<(i32, i32)> {
    let mut spans = Vec::new();
    let mut line_start = 0usize;
    for i in 0..=w.len() {
        if i == w.len() || w[i] == b'\n' as u16 {
            let mut end = i;
            if end > line_start && w[end - 1] == b'\r' as u16 {
                end -= 1;
            }
            if end > line_start {
                spans.push((line_start as i32, end as i32));
            }
            line_start = i + 1;
        }
    }
    spans
}

fn apply_desc_bold_headers(hwnd: HWND, wide: &[u16], lang: Lang) {
    unsafe {
        for (start, end) in utf16_line_spans(wide) {
            let line = String::from_utf16_lossy(&wide[start as usize..end as usize]);
            if !i18n::is_section_header(lang, &line) {
                continue;
            }
            let mut range = CHARRANGE {
                cpMin: start,
                cpMax: end,
            };
            SendMessageW(
                hwnd,
                EM_EXSETSEL,
                0,
                &mut range as *mut CHARRANGE as LPARAM,
            );
            let mut cf: CHARFORMAT2W = std::mem::zeroed();
            cf.cbSize = CHARFORMAT2W_SIZE;
            cf.dwMask = CFM_BOLD | CFM_WEIGHT;
            cf.dwEffects = CFE_BOLD;
            cf.wWeight = FW_BOLD;
            SendMessageW(
                hwnd,
                EM_SETCHARFORMAT,
                SCF_SELECTION,
                &cf as *const CHARFORMAT2W as LPARAM,
            );
        }
        let mut caret = CHARRANGE { cpMin: 0, cpMax: 0 };
        SendMessageW(
            hwnd,
            EM_EXSETSEL,
            0,
            &mut caret as *mut CHARRANGE as LPARAM,
        );
    }
}

fn set_desc_text(hwnd: HWND, text: &str, lang: Lang) {
    let formatted = newlines_for_edit(text);
    let wide = to_wide(&formatted);
    set_text(hwnd, &formatted);
    apply_desc_bold_headers(hwnd, &wide, lang);
}

fn setting_label(s: &SettingRef, lang: Lang) -> String {
    catalog::lookup(&s.setting_str)
        .map(|e| e.title(lang).to_string())
        .unwrap_or_else(|| s.system_name.clone())
}

fn subgroup_label(n: &SubgroupNode, lang: Lang) -> String {
    // Strict UI language: catalog title first; OS name only if catalog has no entry.
    catalog_subgroups::lookup(&n.guid_str)
        .map(|e| e.title(lang).to_string())
        .unwrap_or_else(|| {
            if !n.system_name.is_empty() {
                n.system_name.clone()
            } else {
                catalog_subgroups::title_or_fallback(&n.guid_str, lang)
            }
        })
}

fn create_control(
    class: &str,
    text: &str,
    style: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    parent: HWND,
    id: i32,
    instance: HINSTANCE,
) -> HWND {
    let cls = to_wide(class);
    let tit = to_wide(text);
    unsafe {
        CreateWindowExW(
            0,
            cls.as_ptr(),
            tit.as_ptr(),
            style | WS_CHILD | WS_VISIBLE | WS_TABSTOP,
            x,
            y,
            w,
            h,
            parent,
            id as HMENU,
            instance,
            ptr::null_mut(),
        )
    }
}

fn apply_font(state: &AppState) {
    for h in [
        state.tree,
        state.title,
        state.desc,
        state.vis_label,
        state.vis_mode,
        state.lang_label,
        state.lang_combo,
    ] {
        unsafe {
            SendMessageW(h, WM_SETFONT, state.font as WPARAM, 1);
        }
    }
}

fn refill_vis_combo(state: &mut AppState) {
    let t = i18n::ui(state.lang);
    let prev = state
        .selected_index
        .and_then(|i| state.settings_flat.get(i))
        .map(|s| mode_from_attributes(s.attributes));
    state.suppress_vis_apply = true;
    unsafe {
        SendMessageW(state.vis_mode, CB_RESETCONTENT, 0, 0);
    }
    combo_add_item(state.vis_mode, t.vis_on);
    combo_add_item(state.vis_mode, t.vis_off);
    combo_add_item(state.vis_mode, t.vis_auto);
    if let Some(mode) = prev {
        unsafe {
            SendMessageW(state.vis_mode, CB_SETCURSEL, mode.combo_index(), 0);
        }
    } else {
        unsafe {
            SendMessageW(state.vis_mode, CB_SETCURSEL, 0xFFFF_FFFF, 0);
        }
    }
    state.suppress_vis_apply = false;
}

fn refill_lang_combo(state: &mut AppState) {
    state.suppress_lang_apply = true;
    unsafe {
        SendMessageW(state.lang_combo, CB_RESETCONTENT, 0, 0);
    }
    // Labels stay bilingual so the switcher is always recognizable.
    combo_add_item(state.lang_combo, "Русский");
    combo_add_item(state.lang_combo, "English");
    unsafe {
        SendMessageW(
            state.lang_combo,
            CB_SETCURSEL,
            state.lang.combo_index(),
            0,
        );
    }
    state.suppress_lang_apply = false;
}

fn apply_language_chrome(state: &mut AppState) {
    let t = i18n::ui(state.lang);
    set_text(state.hwnd, t.window_title);
    set_text(state.vis_label, t.vis_label);
    set_text(state.lang_label, t.lang_label);
    refill_vis_combo(state);
    refill_lang_combo(state);
}

fn combo_add_item(combo: HWND, text: &str) {
    let w = to_wide(text);
    unsafe {
        SendMessageW(combo, CB_ADDSTRING, 0, w.as_ptr() as LPARAM);
    }
}

fn set_vis_mode_combo(state: &mut AppState, mode: VisibilityMode) {
    state.suppress_vis_apply = true;
    unsafe {
        SendMessageW(
            state.vis_mode,
            CB_SETCURSEL,
            mode.combo_index(),
            0,
        );
        EnableWindow(state.vis_label, TRUE);
        EnableWindow(state.vis_mode, TRUE);
    }
    state.suppress_vis_apply = false;
}

fn set_vis_mode_disabled(state: &mut AppState) {
    state.suppress_vis_apply = true;
    unsafe {
        SendMessageW(state.vis_mode, CB_SETCURSEL, 0xFFFF_FFFF, 0);
        EnableWindow(state.vis_label, FALSE);
        EnableWindow(state.vis_mode, FALSE);
    }
    state.suppress_vis_apply = false;
}

fn rebuild_tree(state: &mut AppState) {
    // TVM_DELETEITEM + TVI_ROOT очищает всё дерево (отдельного DELETEALLITEMS нет).
    unsafe {
        SendMessageW(state.tree, TVM_DELETEITEM, 0, TVI_ROOT);
    }
    state.settings_flat.clear();
    state.tree_item_index.clear();
    state.tree_group_index.clear();
    state.root_item = 0;

    let root_name = i18n::ui(state.lang).root_label.to_string();
    let root_label = to_wide(&root_name);
    let root_item = TVITEMW {
        mask: TVIF_TEXT | TVIF_CHILDREN,
        pszText: root_label.as_ptr() as *mut u16,
        cchTextMax: -1,
        cChildren: 1,
        ..unsafe { std::mem::zeroed() }
    };
    let mut ins = TVINSERTSTRUCTW {
        hParent: TVI_ROOT,
        hInsertAfter: TVI_LAST,
        item: root_item,
    };
    let root = unsafe { SendMessageW(state.tree, TVM_INSERTITEMW, 0, &mut ins as *mut _ as LPARAM) };
    state.root_item = root as isize;

    let lang = state.lang;
    for (node_idx, node) in state.nodes.iter().enumerate() {
        if node.settings.is_empty() {
            continue;
        }
        let sg_title = subgroup_label(node, lang);
        let mut sg_w = to_wide(&sg_title);
        let sg_item = TVITEMW {
            mask: TVIF_TEXT | TVIF_CHILDREN | TVIF_PARAM,
            pszText: sg_w.as_mut_ptr(),
            cchTextMax: -1,
            cChildren: 1,
            // Отрицательный lParam: раздел (не путать с параметрами, где lParam >= 1).
            lParam: -((node_idx + 1) as isize),
            ..unsafe { std::mem::zeroed() }
        };
        let mut sg_ins = TVINSERTSTRUCTW {
            hParent: root,
            hInsertAfter: TVI_LAST,
            item: sg_item,
        };
        let sg_h =
            unsafe { SendMessageW(state.tree, TVM_INSERTITEMW, 0, &mut sg_ins as *mut _ as LPARAM) }
                as isize;
        if sg_h != 0 {
            state.tree_group_index.push((sg_h, node_idx));
        }

        for s in &node.settings {
            let idx = state.settings_flat.len();
            state.settings_flat.push(s.clone());
            let leaf_title = setting_label(s, lang);
            let mut leaf_w = to_wide(&leaf_title);
            let leaf = TVITEMW {
                mask: TVIF_TEXT | TVIF_PARAM,
                pszText: leaf_w.as_mut_ptr(),
                cchTextMax: -1,
                lParam: (idx + 1) as isize,
                ..unsafe { std::mem::zeroed() }
            };
            let mut leaf_ins = TVINSERTSTRUCTW {
                hParent: sg_h,
                hInsertAfter: TVI_LAST,
                item: leaf,
            };
            let h_leaf = unsafe {
                SendMessageW(state.tree, TVM_INSERTITEMW, 0, &mut leaf_ins as *mut _ as LPARAM)
            } as isize;
            if h_leaf != 0 {
                state.tree_item_index.push((h_leaf, idx));
            }
        }
    }
}

/// Читает реестр и перестраивает дерево (вызывается при старте и после записи).
fn refresh_from_registry(state: &mut AppState) {
    let t = i18n::ui(state.lang);
    match scan_all() {
        Ok(nodes) => {
            state.nodes = nodes;
            rebuild_tree(state);
            state.selected_index = None;
            state.selected_group = None;
            set_text(state.title, t.pick_setting);
            set_desc_text(state.desc, t.list_loaded, state.lang);
            set_vis_mode_disabled(state);
        }
        Err(e) => {
            msg_box(
                state.hwnd,
                &t.registry_read_fail.replace("{code}", &e.to_string()),
                t.error_title,
                MB_OK | MB_ICONWARNING,
            );
        }
    }
}

fn resync_setting(state: &mut AppState, index: usize) {
    let key = (
        state.settings_flat[index].setting_str.clone(),
        state.settings_flat[index].subgroup_str.clone(),
    );
    if let Ok(nodes) = scan_all() {
        state.nodes = nodes;
        if let Some(fresh) = state
            .nodes
            .iter()
            .flat_map(|n| n.settings.iter())
            .find(|x| x.setting_str == key.0 && x.subgroup_str == key.1)
        {
            state.settings_flat[index] = fresh.clone();
        }
    }
}

fn select_setting(state: &mut AppState, index: usize) {
    state.selected_index = Some(index);
    state.selected_group = None;
    let s = &state.settings_flat[index];
    let cat = catalog::lookup(&s.setting_str);
    let title = setting_label(s, state.lang);
    let t = i18n::ui(state.lang);
    let body = cat
        .map(|c| c.body(state.lang))
        .unwrap_or(t.no_catalog_body);
    set_text(state.title, &title);
    set_desc_text(state.desc, body, state.lang);
    set_vis_mode_combo(state, mode_from_attributes(s.attributes));
}

fn select_root(state: &mut AppState) {
    state.selected_index = None;
    state.selected_group = None;
    let groups = state
        .nodes
        .iter()
        .filter(|n| !n.settings.is_empty())
        .count();
    let settings = state.settings_flat.len();
    let t = i18n::ui(state.lang);
    set_text(state.title, t.root_label);
    set_desc_text(
        state.desc,
        &t.root_body
            .replace("{groups}", &groups.to_string())
            .replace("{settings}", &settings.to_string()),
        state.lang,
    );
    set_vis_mode_disabled(state);
}

fn select_subgroup(state: &mut AppState, node_idx: usize) {
    state.selected_index = None;
    state.selected_group = Some(node_idx);
    let Some(node) = state.nodes.get(node_idx) else {
        return;
    };
    let title = subgroup_label(node, state.lang);
    let t = i18n::ui(state.lang);
    let hint = catalog_subgroups::lookup(&node.guid_str)
        .map(|e| e.hint(state.lang))
        .unwrap_or(t.group_fallback_hint);
    let count = node.settings.len();
    set_text(state.title, &title);
    set_desc_text(
        state.desc,
        &t.group_body_fmt
            .replace("{hint}", hint)
            .replace("{count}", &count.to_string()),
        state.lang,
    );
    set_vis_mode_disabled(state);
}

#[derive(Clone)]
enum TreeSelectionKey {
    Root,
    Group(String),
    Setting {
        subgroup: String,
        setting: String,
    },
}

struct TreeUiSnapshot {
    root_expanded: bool,
    expanded_groups: Vec<String>,
    selection: TreeSelectionKey,
}

fn tree_item_is_expanded(tree: HWND, h_item: isize) -> bool {
    if h_item == 0 {
        return false;
    }
    let mut item = TVITEMW {
        mask: TVIF_STATE,
        hItem: h_item,
        stateMask: TVIS_EXPANDED,
        ..unsafe { std::mem::zeroed() }
    };
    let ok = unsafe { SendMessageW(tree, TVM_GETITEMW, 0, &mut item as *mut _ as LPARAM) };
    ok != 0 && (item.state & TVIS_EXPANDED) != 0
}

fn tree_expand_item(tree: HWND, h_item: isize) {
    if h_item == 0 {
        return;
    }
    unsafe {
        SendMessageW(tree, TVM_EXPAND, TVE_EXPAND, h_item);
    }
}

fn tree_select_item(tree: HWND, h_item: isize) {
    if h_item == 0 {
        return;
    }
    unsafe {
        SendMessageW(tree, TVM_SELECTITEM, TVGN_CARET as WPARAM, h_item);
        SendMessageW(tree, TVM_ENSUREVISIBLE, 0, h_item);
    }
}

fn capture_tree_ui(state: &AppState) -> TreeUiSnapshot {
    let root_expanded =
        state.root_item != 0 && tree_item_is_expanded(state.tree, state.root_item);
    let mut expanded_groups = Vec::new();
    for (h, idx) in &state.tree_group_index {
        if tree_item_is_expanded(state.tree, *h) {
            if let Some(n) = state.nodes.get(*idx) {
                expanded_groups.push(n.guid_str.clone());
            }
        }
    }

    let selection = if let Some(i) = state.selected_index {
        if let Some(s) = state.settings_flat.get(i) {
            TreeSelectionKey::Setting {
                subgroup: s.subgroup_str.clone(),
                setting: s.setting_str.clone(),
            }
        } else {
            TreeSelectionKey::Root
        }
    } else if let Some(g) = state.selected_group {
        if let Some(n) = state.nodes.get(g) {
            TreeSelectionKey::Group(n.guid_str.clone())
        } else {
            TreeSelectionKey::Root
        }
    } else {
        TreeSelectionKey::Root
    };

    TreeUiSnapshot {
        root_expanded,
        expanded_groups,
        selection,
    }
}

fn restore_tree_ui(state: &mut AppState, snap: &TreeUiSnapshot) {
    if snap.root_expanded {
        tree_expand_item(state.tree, state.root_item);
    }
    for guid in &snap.expanded_groups {
        if let Some((h, _)) = state
            .tree_group_index
            .iter()
            .find(|(_, idx)| state.nodes.get(*idx).map(|n| &n.guid_str) == Some(guid))
        {
            tree_expand_item(state.tree, *h);
        }
    }

    match &snap.selection {
        TreeSelectionKey::Root => {
            select_root(state);
            tree_select_item(state.tree, state.root_item);
        }
        TreeSelectionKey::Group(guid) => {
            if let Some((h, node_idx)) = state
                .tree_group_index
                .iter()
                .find(|(_, idx)| state.nodes.get(*idx).map(|n| &n.guid_str) == Some(guid))
                .map(|(h, i)| (*h, *i))
            {
                tree_expand_item(state.tree, state.root_item);
                select_subgroup(state, node_idx);
                tree_select_item(state.tree, h);
            } else {
                select_root(state);
                tree_select_item(state.tree, state.root_item);
            }
        }
        TreeSelectionKey::Setting { subgroup, setting } => {
            if let Some((h_leaf, idx)) = state.tree_item_index.iter().find_map(|(h, i)| {
                let s = state.settings_flat.get(*i)?;
                if &s.subgroup_str == subgroup && &s.setting_str == setting {
                    Some((*h, *i))
                } else {
                    None
                }
            }) {
                // Раскрыть корень и родительский раздел.
                tree_expand_item(state.tree, state.root_item);
                if let Some((h_g, _)) = state.tree_group_index.iter().find(|(_, ni)| {
                    state
                        .nodes
                        .get(*ni)
                        .map(|n| &n.guid_str == subgroup)
                        .unwrap_or(false)
                }) {
                    tree_expand_item(state.tree, *h_g);
                }
                select_setting(state, idx);
                tree_select_item(state.tree, h_leaf);
            } else {
                select_root(state);
                tree_select_item(state.tree, state.root_item);
            }
        }
    }
}

fn apply_language(state: &mut AppState, lang: Lang) {
    if state.lang == lang {
        return;
    }
    let snap = capture_tree_ui(state);
    state.lang = lang;
    i18n::save_lang(lang);
    apply_language_chrome(state);
    rebuild_tree(state);
    restore_tree_ui(state, &snap);
}

fn apply_vis_mode_from_combo(state: &mut AppState) {
    let Some(idx) = state.selected_index else {
        return;
    };
    let sel = unsafe { SendMessageW(state.vis_mode, CB_GETCURSEL, 0, 0) };
    let Some(mode) = VisibilityMode::from_combo_index(sel as isize) else {
        return;
    };
    let s = state.settings_flat[idx].clone();
    let prev = mode_from_attributes(s.attributes);
    if mode == prev {
        return;
    }
    if let Err(e) = apply_mode(&s.subgroup, &s.setting, &s.registry_path, mode) {
        let t = i18n::ui(state.lang);
        msg_box(
            state.hwnd,
            &t.registry_write_fail.replace("{code}", &e.to_string()),
            t.error_title,
            MB_OK | MB_ICONWARNING,
        );
        set_vis_mode_combo(state, prev);
        return;
    }
    resync_setting(state, idx);
    if let Some(fresh) = state.settings_flat.get(idx) {
        set_vis_mode_combo(state, mode_from_attributes(fresh.attributes));
    }
}

/// В `TVN_SELCHANGED` поле `lParam` в уведомлении часто пустое — читаем через `TVM_GETITEM`.
fn tree_caret_item(tree: HWND) -> isize {
    unsafe { SendMessageW(tree, TVM_GETNEXTITEM, TVGN_CARET as WPARAM, 0) as isize }
}

fn tree_hit_item(tree: HWND, x: i32, y: i32) -> isize {
    let mut ht = TVHITTESTINFO {
        pt: POINT { x, y },
        flags: 0,
        hItem: 0,
    };
    unsafe { SendMessageW(tree, TVM_HITTEST, 0, &mut ht as *mut _ as LPARAM) };
    ht.hItem
}

fn sync_panel_from_tree(hwnd: HWND, h_item_hint: isize) {
    with_state(hwnd, |state| {
        let mut h = h_item_hint;
        if h == 0 {
            h = tree_caret_item(state.tree);
        }
        apply_tree_selection(state, h);
    });
}

fn request_tree_sync(parent: HWND, h_item_hint: isize) {
    // PostMessage: после полной обработки клавиши TreeView уже сменит caret.
    unsafe {
        PostMessageW(parent, WM_TREE_SYNC, h_item_hint as WPARAM, 0);
    }
}

fn tree_mouse_item(hwnd: HWND, lparam: LPARAM) -> isize {
    let x = (lparam & 0xFFFF) as u16 as i16 as i32;
    let y = ((lparam >> 16) & 0xFFFF) as u16 as i16 as i32;
    tree_hit_item(hwnd, x, y)
}

/// Предыдущий WndProc дерева (классический subclass — надёжнее для стрелок, чем SetWindowSubclass).
static mut TREE_PREV_WNDPROC: isize = 0;

unsafe extern "system" fn tree_wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let parent = GetParent(hwnd);
    let prev = TREE_PREV_WNDPROC;
    match msg {
        WM_LBUTTONDOWN => {
            let h = tree_mouse_item(hwnd, lparam);
            let r = CallWindowProcW(prev, hwnd, msg, wparam, lparam);
            request_tree_sync(parent, h);
            return r;
        }
        WM_KEYDOWN | WM_KEYUP if is_tree_nav_key(wparam) => {
            let r = CallWindowProcW(prev, hwnd, msg, wparam, lparam);
            let caret = tree_caret_item(hwnd);
            request_tree_sync(parent, caret);
            return r;
        }
        _ => {}
    }
    CallWindowProcW(prev, hwnd, msg, wparam, lparam)
}

fn tree_item_label(tree: HWND, h_item: isize) -> Option<String> {
    if h_item == 0 {
        return None;
    }
    let mut buf = [0u16; 512];
    let mut item = TVITEMW {
        mask: TVIF_TEXT,
        hItem: h_item,
        pszText: buf.as_mut_ptr(),
        cchTextMax: buf.len() as i32,
        ..unsafe { std::mem::zeroed() }
    };
    let ok = unsafe { SendMessageW(tree, TVM_GETITEMW, 0, &mut item as *mut _ as LPARAM) };
    if ok == 0 {
        return None;
    }
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..end]))
}

fn item_lparam(tree: HWND, h_item: isize) -> Option<isize> {
    if h_item == 0 {
        return None;
    }
    let mut item = TVITEMW {
        mask: TVIF_PARAM,
        hItem: h_item,
        ..unsafe { std::mem::zeroed() }
    };
    let ok = unsafe { SendMessageW(tree, TVM_GETITEMW, 0, &mut item as *mut _ as LPARAM) };
    if ok == 0 {
        None
    } else {
        Some(item.lParam)
    }
}

fn setting_index_for_item(state: &AppState, h_item: isize) -> Option<usize> {
    // Только явный lParam параметра (>= 1). Не угадывать по тексту — иначе
    // раздел с похожим названием перехватится как параметр.
    if let Some(lp) = item_lparam(state.tree, h_item) {
        if lp >= 1 {
            let idx = (lp - 1) as usize;
            if idx < state.settings_flat.len() {
                return Some(idx);
            }
        }
    }
    state
        .tree_item_index
        .iter()
        .find(|(h, _)| *h == h_item)
        .map(|(_, idx)| *idx)
}

fn group_index_for_item(state: &AppState, h_item: isize) -> Option<usize> {
    if let Some((_, idx)) = state.tree_group_index.iter().find(|(h, _)| *h == h_item) {
        return Some(*idx);
    }
    // lParam раздела: -(node_idx + 1)
    if let Some(lp) = item_lparam(state.tree, h_item) {
        if lp < 0 {
            let idx = ((-lp) as usize).saturating_sub(1);
            if idx < state.nodes.len() {
                return Some(idx);
            }
        }
    }
    let label = tree_item_label(state.tree, h_item)?;
    let lang = state.lang;
    state
        .nodes
        .iter()
        .position(|n| !n.settings.is_empty() && subgroup_label(n, lang) == label)
}

fn apply_tree_selection(state: &mut AppState, h_item: isize) {
    if h_item == 0 {
        return;
    }
    if state.root_item != 0 && h_item == state.root_item {
        select_root(state);
        return;
    }
    // Сначала раздел (отрицательный lParam / карта групп), потом параметр.
    if let Some(node_idx) = group_index_for_item(state, h_item) {
        select_subgroup(state, node_idx);
        return;
    }
    if let Some(idx) = setting_index_for_item(state, h_item) {
        if idx < state.settings_flat.len() {
            select_setting(state, idx);
            return;
        }
    }
    // Неизвестный узел — всё равно обновим панель, а не оставим старое описание.
    state.selected_index = None;
    state.selected_group = None;
    let t = i18n::ui(state.lang);
    if let Some(label) = tree_item_label(state.tree, h_item) {
        set_text(state.title, &label);
        set_desc_text(state.desc, t.unknown_item_body, state.lang);
    }
    set_vis_mode_disabled(state);
}

fn notify_code_i32(code: UINT) -> i32 {
    code as i32
}

fn is_selchanged_notify(code: UINT) -> bool {
    let c = notify_code_i32(code);
    c == TVN_SELCHANGEDW || c == TVN_SELCHANGEDA
}

fn tree_notify_needs_sync(code: UINT) -> bool {
    let c = notify_code_i32(code);
    is_selchanged_notify(code) || c == NM_CLICK || c == TVN_KEYDOWN
}

fn layout(state: &AppState) {
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        GetClientRect(state.hwnd, &mut rc);
    }
    let w = rc.right - rc.left;
    let h = rc.bottom - rc.top;
    let margin = 10;
    let tree_w = (w * 40 / 100).max(220);
    let right_x = margin + tree_w + margin;
    let right_w = w - right_x - margin;
    let row_h = 28;
    let lang_label_w = 70;
    let lang_combo_w = 110;
    let vis_label_w = 220.min(right_w.saturating_sub(130));
    let vis_combo_w = (right_w - vis_label_w - 8).clamp(100, 160);
    let bottom_y = h - margin - row_h;
    let lang_y = bottom_y - row_h - 8;
    let desc_bottom = lang_y - 8;
    let desc_top = margin + 36;
    let desc_h = (desc_bottom - desc_top).max(80);

    unsafe {
        MoveWindow(
            state.tree,
            margin,
            margin,
            tree_w,
            h - 2 * margin,
            TRUE,
        );
        MoveWindow(state.title, right_x, margin, right_w, 28, TRUE);
        MoveWindow(state.desc, right_x, desc_top, right_w, desc_h, TRUE);
        MoveWindow(
            state.lang_label,
            right_x,
            lang_y,
            lang_label_w,
            row_h,
            TRUE,
        );
        MoveWindow(
            state.lang_combo,
            right_x + lang_label_w + 8,
            lang_y,
            lang_combo_w,
            row_h + 200,
            TRUE,
        );
        MoveWindow(
            state.vis_label,
            right_x,
            bottom_y,
            vis_label_w,
            row_h,
            TRUE,
        );
        MoveWindow(
            state.vis_mode,
            right_x + vis_label_w + 8,
            bottom_y,
            vis_combo_w,
            row_h + 200,
            TRUE,
        );
    }
}

#[link(name = "user32")]
extern "system" {
    fn EnableWindow(hwnd: HWND, enable: BOOL) -> BOOL;
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create = &*(lparam as *const CREATESTRUCTW);
            let instance = create.hInstance;
            let font = CreateFontW(
                -11,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                0,
                0,
                0,
                0,
                to_wide("Segoe UI").as_ptr(),
            );
            let tree = create_control(
                WC_TREEVIEWW,
                "",
                TVS_HASLINES | TVS_HASBUTTONS | TVS_LINESATROOT | TVS_SHOWSELALWAYS | WS_BORDER,
                0,
                0,
                100,
                100,
                hwnd,
                ID_TREE,
                instance,
            );
            let lang = i18n::initial_lang();
            let t = i18n::ui(lang);
            let title = create_control(
                WC_STATICW,
                t.loading,
                0,
                0,
                0,
                100,
                20,
                hwnd,
                ID_TITLE,
                instance,
            );
            let desc = create_control(
                WC_RICHEDIT20W,
                "",
                ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL | WS_VSCROLL,
                0,
                0,
                100,
                100,
                hwnd,
                ID_DESC,
                instance,
            );
            let lang_label = create_control(
                WC_STATICW,
                t.lang_label,
                0,
                0,
                0,
                80,
                24,
                hwnd,
                ID_LANG_LABEL,
                instance,
            );
            let lang_combo = create_control(
                WC_COMBOBOXW,
                "",
                CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_VSCROLL,
                0,
                0,
                120,
                200,
                hwnd,
                ID_LANG,
                instance,
            );
            let vis_label = create_control(
                WC_STATICW,
                t.vis_label,
                0,
                0,
                0,
                200,
                24,
                hwnd,
                ID_VIS_LABEL,
                instance,
            );
            let vis_mode = create_control(
                WC_COMBOBOXW,
                "",
                CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_VSCROLL,
                0,
                0,
                120,
                200,
                hwnd,
                ID_VIS_MODE,
                instance,
            );

            let mut app = Box::new(AppState {
                hwnd,
                tree,
                title,
                desc,
                vis_label,
                vis_mode,
                lang_label,
                lang_combo,
                lang,
                nodes: Vec::new(),
                settings_flat: Vec::new(),
                tree_item_index: Vec::new(),
                tree_group_index: Vec::new(),
                root_item: 0,
                selected_index: None,
                selected_group: None,
                suppress_vis_apply: false,
                suppress_lang_apply: false,
                font,
            });
            apply_font(&app);
            apply_language_chrome(&mut app);
            set_vis_mode_disabled(&mut app);
            layout(&app);
            unsafe {
                TREE_PREV_WNDPROC = SetWindowLongPtrW(
                    tree,
                    GWLP_WNDPROC,
                    tree_wnd_proc as usize as isize,
                );
            }

            let ptr = Box::into_raw(app);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);
            refresh_from_registry(unsafe { &mut *ptr });
            return 0;
        }
        WM_TREE_SYNC => {
            sync_panel_from_tree(hwnd, wparam as isize);
            return 0;
        }
        WM_SIZE => {
            with_state(hwnd, |state| layout(state));
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        WM_GETMINMAXINFO => {
            let info = &mut *(lparam as *mut MINMAXINFO);
            info.ptMinTrackSize.x = 680;
            info.ptMinTrackSize.y = 420;
            return 0;
        }
        WM_NOTIFY => {
            let nm = &*(lparam as *const NMHDR);
            let tree = with_state(hwnd, |state| state.tree);
            if nm.hwndFrom == tree && tree_notify_needs_sync(nm.code) {
                // caret читаем уже в WM_TREE_SYNC — к этому моменту выделение актуально.
                request_tree_sync(hwnd, 0);
            }
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            let code = ((wparam >> 16) & 0xFFFF) as u16;
            if id == ID_VIS_MODE && code == CBN_SELCHANGE {
                with_state(hwnd, |state| {
                    if state.suppress_vis_apply {
                        return;
                    }
                    apply_vis_mode_from_combo(state);
                });
            } else if id == ID_LANG && code == CBN_SELCHANGE {
                with_state(hwnd, |state| {
                    if state.suppress_lang_apply {
                        return;
                    }
                    let sel = unsafe { SendMessageW(state.lang_combo, CB_GETCURSEL, 0, 0) };
                    if let Some(lang) = Lang::from_combo_index(sel as isize) {
                        apply_language(state, lang);
                    }
                });
            }
            return 0;
        }
        WM_DESTROY => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if ptr != 0 {
                let b = unsafe { Box::from_raw(ptr as *mut AppState) };
                unsafe {
                    DeleteObject(b.font);
                }
                drop(b);
            }
            PostQuitMessage(0);
            return 0;
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[repr(C)]
struct CREATESTRUCTW {
    lpCreateParams: *mut c_void,
    hInstance: HINSTANCE,
    hMenu: HMENU,
    hwndParent: HWND,
    cy: i32,
    cx: i32,
    y: i32,
    x: i32,
    style: i32,
    lpszName: *const u16,
    lpszClass: *const u16,
    dwExStyle: u32,
}

pub fn run() -> i32 {
    unsafe {
        let riched = to_wide("riched20.dll");
        if LoadLibraryW(riched.as_ptr()) == 0 {
            let t = i18n::ui(i18n::initial_lang());
            msg_box(0, t.richedit_fail, t.error_title, MB_OK | MB_ICONERROR);
            return 1;
        }
        let ic = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_TREEVIEW_CLASSES | ICC_STANDARD_CLASSES,
        };
        InitCommonControlsEx(&ic);
    }

    let instance = unsafe { GetModuleHandleW(ptr::null()) };
    let class_name = to_wide("PowerSettingsExplorerWnd");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: 0,
        hCursor: unsafe { LoadCursorW(0 as HINSTANCE, IDC_ARROW) },
        hbrBackground: unsafe { GetStockObject(COLOR_WINDOW + 1) as isize },
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: 0,
    };
    unsafe {
        RegisterClassExW(&wc);
    }

    let title = to_wide(i18n::ui(i18n::initial_lang()).window_title);
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            100,
            100,
            920,
            600,
            0,
            0,
            instance,
            ptr::null_mut(),
        )
    };
    if hwnd == 0 {
        return 1;
    }
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    let mut msg = MSG {
        hwnd: 0,
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: POINT { x: 0, y: 0 },
    };
    loop {
        let ret = unsafe { GetMessageW(&mut msg, 0, 0, 0) };
        if ret == 0 || ret == -1 {
            break;
        }
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    0
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> HINSTANCE;
}
