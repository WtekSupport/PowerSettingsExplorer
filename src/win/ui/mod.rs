mod win32;

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

const WM_TREE_SYNC: UINT = 0x8000 + 42;
const WM_LBUTTONDOWN: UINT = 0x0201;
const WM_LBUTTONUP: UINT = 0x0202;
const WM_KEYUP: UINT = 0x0101;
const VK_UP: WPARAM = 0x26;
const VK_DOWN: WPARAM = 0x28;
const VK_HOME: WPARAM = 0x24;
const VK_END: WPARAM = 0x23;
const VK_PRIOR: WPARAM = 0x21;
const VK_NEXT: WPARAM = 0x22;

struct AppState {
    hwnd: HWND,
    tree: HWND,
    title: HWND,
    desc: HWND,
    vis_label: HWND,
    vis_mode: HWND,
    nodes: Vec<SubgroupNode>,
    settings_flat: Vec<SettingRef>,
    /// HTREEITEM → индекс в `settings_flat` (надёжнее, чем lParam дерева).
    tree_item_index: Vec<(isize, usize)>,
    selected_index: Option<usize>,
    /// Подавляет запись в реестр при программной смене списка.
    suppress_vis_apply: bool,
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

fn is_desc_section_header(line: &str) -> bool {
    let t = line.trim();
    matches!(t, "Назначение" | "Поведение" | "На практике")
        || t.starts_with("Где задать")
        || t.starts_with("Видимость")
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

fn apply_desc_bold_headers(hwnd: HWND, wide: &[u16]) {
    unsafe {
        for (start, end) in utf16_line_spans(wide) {
            let line = String::from_utf16_lossy(&wide[start as usize..end as usize]);
            if !is_desc_section_header(&line) {
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

fn set_desc_text(hwnd: HWND, text: &str) {
    let formatted = newlines_for_edit(text);
    let wide = to_wide(&formatted);
    set_text(hwnd, &formatted);
    apply_desc_bold_headers(hwnd, &wide);
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
    ] {
        unsafe {
            SendMessageW(h, WM_SETFONT, state.font as WPARAM, 1);
        }
    }
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
    unsafe {
        SendMessageW(state.tree, TVM_DELETEALLITEMS, 0, 0);
    }
    state.settings_flat.clear();
    state.tree_item_index.clear();

    let root_label = to_wide("Параметры электропитания");
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

    for node in &state.nodes {
        if node.settings.is_empty() {
            continue;
        }
        let sg_title = catalog_subgroups::title_or_fallback(&node.guid_str);
        let mut sg_w = to_wide(&sg_title);
        let sg_item = TVITEMW {
            mask: TVIF_TEXT | TVIF_CHILDREN,
            pszText: sg_w.as_mut_ptr(),
            cchTextMax: -1,
            cChildren: 1,
            ..unsafe { std::mem::zeroed() }
        };
        let mut sg_ins = TVINSERTSTRUCTW {
            hParent: root,
            hInsertAfter: TVI_LAST,
            item: sg_item,
        };
        let sg_h =
            unsafe { SendMessageW(state.tree, TVM_INSERTITEMW, 0, &mut sg_ins as *mut _ as LPARAM) };

        for s in &node.settings {
            let idx = state.settings_flat.len();
            state.settings_flat.push(s.clone());
            let title = catalog::lookup(&s.setting_str)
                .map(|e| e.title_ru.to_string())
                .unwrap_or_else(|| s.setting_str.clone());
            let mut leaf_w = to_wide(&title);
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
    match scan_all() {
        Ok(nodes) => {
            state.nodes = nodes;
            rebuild_tree(state);
            state.selected_index = None;
            set_text(state.title, "Выберите параметр слева");
            set_desc_text(
                state.desc,
                "Список загружен из реестра Windows.\n\n\
                 Выберите пункт слева — справа появятся описание блоками и селектор видимости (Вкл / Выкл / Авто).",
            );
            set_vis_mode_disabled(state);
        }
        Err(e) => {
            msg_box(
                state.hwnd,
                &format!(
                    "Не удалось прочитать параметры из реестра (код {e}).\nЗапустите программу от имени администратора."
                ),
                "Ошибка",
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
    let s = &state.settings_flat[index];
    let cat = catalog::lookup(&s.setting_str);
    let title = cat.map(|c| c.title_ru).unwrap_or("Параметр электропитания");
    let body = cat.map(|c| c.body_ru).unwrap_or(
        "Назначение\nОписание для этого GUID пока не добавлено в каталог.\n\n\
         Где задать «От сети» / «От батареи»\n\
         Панель управления → Электропитание → Настройка схемы → Изменить дополнительные параметры.\n\n\
         Видимость в этой программе\n\
         Селектор «Вкл / Выкл / Авто» ниже меняет только видимость пункта в списке, не сами значения.",
    );
    set_text(state.title, title);
    set_desc_text(state.desc, body);
    set_vis_mode_combo(state, mode_from_attributes(s.attributes));
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
        msg_box(
            state.hwnd,
            &format!(
                "Не удалось записать в реестр (код {e}).\nНужен запуск от администратора."
            ),
            "Ошибка",
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

fn tree_item_stored_index(tree: HWND, h_item: isize) -> Option<usize> {
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
        return None;
    }
    let lp = item.lParam;
    if lp >= 1 {
        Some((lp - 1) as usize)
    } else {
        None
    }
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

fn tree_mouse_item(hwnd: HWND, lparam: LPARAM) -> isize {
    let x = (lparam & 0xFFFF) as u16 as i16 as i32;
    let y = ((lparam >> 16) & 0xFFFF) as u16 as i16 as i32;
    tree_hit_item(hwnd, x, y)
}

unsafe extern "system" fn tree_subclass_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    parent: usize,
) -> LRESULT {
    let parent = parent as HWND;
    if msg == WM_LBUTTONDOWN {
        let h = tree_mouse_item(hwnd, lparam);
        SendMessageW(parent, WM_TREE_SYNC, h as WPARAM, 0);
    }
    let r = DefSubclassProc(hwnd, msg, wparam, lparam);
    match msg {
        WM_KEYUP if matches!(
            wparam,
            VK_UP | VK_DOWN | VK_HOME | VK_END | VK_PRIOR | VK_NEXT
        ) =>
        {
            SendMessageW(parent, WM_TREE_SYNC, 0, 0);
        }
        _ => {}
    }
    r
}

fn display_title_for(s: &SettingRef) -> String {
    catalog::lookup(&s.setting_str)
        .map(|e| e.title_ru.to_string())
        .unwrap_or_else(|| s.setting_str.clone())
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

fn setting_index_for_item(state: &AppState, h_item: isize) -> Option<usize> {
    if let Some(idx) = tree_item_stored_index(state.tree, h_item) {
        if idx < state.settings_flat.len() {
            return Some(idx);
        }
    }
    if let Some((_, idx)) = state
        .tree_item_index
        .iter()
        .find(|(h, _)| *h == h_item)
    {
        return Some(*idx);
    }
    let label = tree_item_label(state.tree, h_item)?;
    state
        .settings_flat
        .iter()
        .position(|s| display_title_for(s) == label)
}

fn apply_tree_selection(state: &mut AppState, h_item: isize) {
    if h_item == 0 {
        return;
    }
    if let Some(idx) = setting_index_for_item(state, h_item) {
        if idx < state.settings_flat.len() {
            select_setting(state, idx);
            return;
        }
    }
    state.selected_index = None;
    set_text(state.title, "Выберите параметр слева");
    set_desc_text(
        state.desc,
        "Выберите конкретный параметр (не группу).\n\n\
         Справа — описание по блокам и режим видимости (Вкл / Выкл / Авто).",
    );
    set_vis_mode_disabled(state);
}

fn is_selchanged_notify(code: UINT) -> bool {
    let c = code as i32;
    c == TVN_SELCHANGEDW || c == TVN_SELCHANGEDA
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
    let vis_row_h = 28;
    let label_w = 220.min(right_w.saturating_sub(130));
    let combo_w = (right_w - label_w - 8).clamp(100, 160);

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
        MoveWindow(
            state.desc,
            right_x,
            margin + 36,
            right_w,
            h - 2 * margin - 36 - vis_row_h - 8,
            TRUE,
        );
        MoveWindow(
            state.vis_label,
            right_x,
            h - margin - vis_row_h,
            label_w,
            vis_row_h,
            TRUE,
        );
        MoveWindow(
            state.vis_mode,
            right_x + label_w + 8,
            h - margin - vis_row_h,
            combo_w,
            vis_row_h + 200,
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
            let title = create_control(
                WC_STATICW,
                "Загрузка списка…",
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
            let vis_label = create_control(
                WC_STATICW,
                "Показ в доп. параметрах:",
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
            combo_add_item(vis_mode, "Вкл");
            combo_add_item(vis_mode, "Выкл");
            combo_add_item(vis_mode, "Авто");

            let mut app = Box::new(AppState {
                hwnd,
                tree,
                title,
                desc,
                vis_label,
                vis_mode,
                nodes: Vec::new(),
                settings_flat: Vec::new(),
                tree_item_index: Vec::new(),
                selected_index: None,
                suppress_vis_apply: false,
                font,
            });
            apply_font(&app);
            set_vis_mode_disabled(&mut app);
            layout(&app);
            unsafe {
                SetWindowSubclass(tree, tree_subclass_proc, 1, hwnd as usize);
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
            with_state(hwnd, |state| {
                if nm.hwndFrom != state.tree {
                    return;
                }
                if !is_selchanged_notify(nm.code) && nm.code as i32 != NM_CLICK {
                    return;
                }
                sync_panel_from_tree(hwnd, 0);
            });
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
            msg_box(
                0,
                "Не удалось загрузить riched20.dll (Rich Edit).",
                "Ошибка",
                MB_OK | MB_ICONERROR,
            );
            return 1;
        }
        let ic = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_TREEVIEW_CLASSES | ICC_STANDARD_CLASSES,
        };
        InitCommonControlsEx(&ic);
    }

    let instance = unsafe { GetModuleHandleW(ptr::null()) };
    let class_name = to_wide("PowerSettingsVisibilityWnd");
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

    let title = to_wide("Параметры электропитания — видимость");
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
