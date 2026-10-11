#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::c_void;

pub type HWND = isize;
pub type HINSTANCE = isize;
pub type HMENU = isize;
pub type HFONT = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type UINT = u32;
pub type BOOL = i32;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

pub const WM_CREATE: UINT = 0x0001;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_SHOWWINDOW: UINT = 0x0018;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_NOTIFY: UINT = 0x004E;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_GETMINMAXINFO: UINT = 0x0024;

pub const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
pub const WS_VISIBLE: u32 = 0x10000000;
pub const WS_CHILD: u32 = 0x40000000;
pub const WS_BORDER: u32 = 0x00800000;
pub const WS_VSCROLL: u32 = 0x00200000;
pub const WS_TABSTOP: u32 = 0x00010000;

pub const BS_PUSHBUTTON: u32 = 0;
pub const BS_AUTOCHECKBOX: u32 = 0x00000003;
pub const BS_PUSHLIKE: u32 = 0x00001000;
pub const BS_NOTIFY: u32 = 0x00010000;

pub const WC_COMBOBOXW: &str = "ComboBox";
pub const CBS_DROPDOWNLIST: u32 = 0x0003;
pub const CBS_HASSTRINGS: u32 = 0x0200;
pub const CB_ADDSTRING: UINT = 0x0143;
pub const CB_RESETCONTENT: UINT = 0x014B;
pub const CB_SETCURSEL: UINT = 0x014E;
pub const CB_GETCURSEL: UINT = 0x0147;
pub const CBN_SELCHANGE: u16 = 1;

pub const ES_MULTILINE: u32 = 0x0004;
pub const ES_READONLY: u32 = 0x0800;
pub const ES_AUTOVSCROLL: u32 = 0x0040;

pub const WC_RICHEDIT20W: &str = "RichEdit20W";

pub const EM_EXSETSEL: UINT = 0x043B;
pub const EM_SETCHARFORMAT: UINT = 0x0444;
pub const SCF_SELECTION: WPARAM = 1;
pub const CFE_BOLD: u32 = 0x0001;
pub const CFM_BOLD: u32 = 0x00000001;
pub const CFM_WEIGHT: u32 = 0x00004000;
pub const FW_BOLD: u16 = 700;
/// Размер `_CHARFORMAT2W` в Windows SDK (не совпадает с `size_of` нашей Rust-структуры без padding).
pub const CHARFORMAT2W_SIZE: u32 = 124;

#[repr(C)]
pub struct CHARRANGE {
    pub cpMin: i32,
    pub cpMax: i32,
}

/// `_CHARFORMAT2W` — только поля, нужные для жирного начертания.
#[repr(C)]
pub struct CHARFORMAT2W {
    pub cbSize: u32,
    pub dwMask: u32,
    pub dwEffects: u32,
    pub yHeight: i32,
    pub yOffset: i32,
    pub crTextColor: u32,
    pub bCharSet: u8,
    pub bPitchAndFamily: u8,
    pub szFaceName: [u16; 32],
    pub wWeight: u16,
    pub sSpacing: i16,
    pub crBackColor: u32,
    pub lcid: u32,
    pub dwReserved: u32,
    pub sStyle: i16,
    pub wKerning: u16,
    pub bUnderlineType: u8,
    pub bAnimation: u8,
    pub bRevAuthor: u8,
    pub bReserved1: u8,
    pub _sdk_pad: [u8; 8],
}

pub const TVS_HASLINES: u32 = 0x0004;
pub const TVS_HASBUTTONS: u32 = 0x0001;
pub const TVS_LINESATROOT: u32 = 0x0008;
pub const TVS_SHOWSELALWAYS: u32 = 0x0020;

pub const TV_FIRST: UINT = 0x1100;
pub const TVM_INSERTITEMW: UINT = TV_FIRST + 50;
pub const TVM_GETITEMW: UINT = TV_FIRST + 62;
/// Было ошибочно +13 (= TVM_SETITEMA) — из‑за этого стрелки не читали caret.
pub const TVM_GETNEXTITEM: UINT = TV_FIRST + 10;
pub const TVM_HITTEST: UINT = TV_FIRST + 17;
pub const TVM_SELECTITEM: UINT = TV_FIRST + 11;
pub const TVM_EXPAND: UINT = TV_FIRST + 2;
pub const TVM_ENSUREVISIBLE: UINT = TV_FIRST + 20;
/// Удаление узла. Чтобы очистить всё дерево: `lParam = TVI_ROOT`.
pub const TVM_DELETEITEM: UINT = TV_FIRST + 1;
/// Для `TVM_GETNEXTITEM` — текущий выбранный элемент.
pub const TVGN_CARET: u32 = 0x0009;

pub const TVE_EXPAND: WPARAM = 0x0002;
pub const TVIF_STATE: u32 = 0x0008;
pub const TVIS_EXPANDED: u32 = 0x0020;

pub const NM_CLICK: i32 = NM_FIRST - 2;

pub const NM_FIRST: i32 = 0;
/// `TVN_FIRST` from commctrl.h (0U-400U)
pub const TVN_FIRST: i32 = -400;
pub const TVN_SELCHANGEDA: i32 = TVN_FIRST - 1;
pub const TVN_SELCHANGEDW: i32 = TVN_FIRST - 2;
pub const TVN_KEYDOWN: i32 = TVN_FIRST - 12;

pub const ICC_TREEVIEW_CLASSES: u32 = 0x00000200;
pub const ICC_STANDARD_CLASSES: u32 = 0x00004000;

pub const DEFAULT_CHARSET: u32 = 1;
pub const FW_NORMAL: i32 = 400;

pub const IDI_APPLICATION: *const u16 = 32512 as *const u16;
pub const IDC_ARROW: *const u16 = 32512 as *const u16;
pub const COLOR_WINDOW: i32 = 5;

pub const GWLP_WNDPROC: i32 = -4;
pub const GWLP_USERDATA: i32 = -21;

pub const MB_OK: u32 = 0;
pub const MB_YESNO: u32 = 0x00000004;
pub const MB_ICONINFORMATION: u32 = 0x00000040;
pub const MB_ICONWARNING: u32 = 0x00000030;
pub const MB_ICONERROR: u32 = 0x00000010;
pub const IDYES: i32 = 6;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: isize,
    pub hCursor: isize,
    pub hbrBackground: isize,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: isize,
}

#[repr(C)]
pub struct INITCOMMONCONTROLSEX {
    pub dwSize: u32,
    pub dwICC: u32,
}

#[repr(C)]
pub struct TVINSERTSTRUCTW {
    pub hParent: isize,
    pub hInsertAfter: isize,
    pub item: TVITEMW,
}

#[repr(C)]
pub struct TVITEMW {
    pub mask: u32,
    pub hItem: isize,
    pub state: u32,
    pub stateMask: u32,
    pub pszText: *mut u16,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub iSelectedImage: i32,
    pub cChildren: i32,
    pub lParam: LPARAM,
}

pub const TVIF_TEXT: u32 = 0x0001;
pub const TVIF_PARAM: u32 = 0x0004;
pub const TVIF_CHILDREN: u32 = 0x0040;

pub const TVI_ROOT: isize = -0x10000;
pub const TVI_LAST: isize = -0x0FFFF;

/// Как в Win32 на x64: hwnd(8) + id(8) + code(4) + pad(4) = 24.
#[repr(C)]
pub struct NMHDR {
    pub hwndFrom: HWND,
    pub idFrom: usize,
    pub code: UINT,
    pub _pad: u32,
}

#[repr(C)]
pub struct NMTREEVIEWW {
    pub hdr: NMHDR,
    pub action: u32,
    pub itemOld: TVITEMW,
    pub itemNew: TVITEMW,
    pub ptDrag: POINT,
}

#[repr(C)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
pub struct TVHITTESTINFO {
    pub pt: POINT,
    pub flags: u32,
    pub hItem: isize,
}

#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct MINMAXINFO {
    pub ptReserved: POINT,
    pub ptMaxSize: POINT,
    pub ptMaxPosition: POINT,
    pub ptMinTrackSize: POINT,
    pub ptMaxTrackSize: POINT,
}

pub type SubclassProc = unsafe extern "system" fn(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    ref_data: usize,
) -> LRESULT;

#[link(name = "comctl32")]
extern "system" {
    pub fn SetWindowSubclass(
        hwnd: HWND,
        pfn: SubclassProc,
        id: usize,
        ref_data: usize,
    ) -> BOOL;
    pub fn DefSubclassProc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}

#[link(name = "user32")]
extern "system" {
    pub fn PostMessageW(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> BOOL;
    pub fn RegisterClassExW(lp: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: HWND,
        menu: HMENU,
        instance: HINSTANCE,
        param: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn GetMessageW(msg: *mut MSG, hwnd: HWND, min: UINT, max: UINT) -> BOOL;
    pub fn TranslateMessage(msg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(msg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(exit: i32);
    pub fn ShowWindow(hwnd: HWND, cmd: i32) -> BOOL;
    pub fn UpdateWindow(hwnd: HWND) -> BOOL;
    pub fn SetWindowTextW(hwnd: HWND, text: *const u16);
    pub fn GetClientRect(hwnd: HWND, rect: *mut RECT) -> BOOL;
    pub fn MoveWindow(hwnd: HWND, x: i32, y: i32, w: i32, h: i32, repaint: BOOL) -> BOOL;
    pub fn SetWindowPos(
        hwnd: HWND,
        hwnd_insert_after: HWND,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> BOOL;
    pub fn SendMessageW(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn SetWindowLongPtrW(hwnd: HWND, index: i32, val: isize) -> isize;
    pub fn GetWindowLongPtrW(hwnd: HWND, index: i32) -> isize;
    pub fn CallWindowProcW(
        prev: isize,
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT;
    pub fn GetParent(hwnd: HWND) -> HWND;
    pub fn MessageBoxW(hwnd: HWND, text: *const u16, caption: *const u16, typ: u32) -> i32;
    pub fn LoadCursorW(instance: HINSTANCE, name: *const u16) -> isize;
    pub fn GetStockObject(obj: i32) -> isize;
}

#[link(name = "comctl32")]
extern "system" {
    pub fn InitCommonControlsEx(ic: *const INITCOMMONCONTROLSEX) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn LoadLibraryW(name: *const u16) -> HINSTANCE;
}

#[link(name = "comdlg32")]
extern "system" {
    pub fn GetOpenFileNameW(p: *mut OPENFILENAMEW) -> BOOL;
    pub fn GetSaveFileNameW(p: *mut OPENFILENAMEW) -> BOOL;
}

pub const OFN_OVERWRITEPROMPT: u32 = 0x00000002;
pub const OFN_PATHMUSTEXIST: u32 = 0x00000800;
pub const OFN_FILEMUSTEXIST: u32 = 0x00001000;
pub const OFN_HIDEREADONLY: u32 = 0x00000004;
pub const OFN_EXPLORER: u32 = 0x00080000;

#[repr(C)]
pub struct OPENFILENAMEW {
    pub lStructSize: u32,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub lpstrFilter: *const u16,
    pub lpstrCustomFilter: *mut u16,
    pub nMaxCustFilter: u32,
    pub nFilterIndex: u32,
    pub lpstrFile: *mut u16,
    pub nMaxFile: u32,
    pub lpstrFileTitle: *mut u16,
    pub nMaxFileTitle: u32,
    pub lpstrInitialDir: *const u16,
    pub lpstrTitle: *const u16,
    pub Flags: u32,
    pub nFileOffset: u16,
    pub nFileExtension: u16,
    pub lpstrDefExt: *const u16,
    pub lCustData: isize,
    pub lpfnHook: *mut c_void,
    pub lpTemplateName: *const u16,
    pub pvReserved: *mut c_void,
    pub dwReserved: u32,
    pub FlagsEx: u32,
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateFontW(
        h: i32,
        w: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike: u32,
        charset: u32,
        out: u32,
        clip: u32,
        quality: u32,
        pitch: u32,
        face: *const u16,
    ) -> HFONT;
    pub fn DeleteObject(obj: isize) -> BOOL;
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}

pub const SW_SHOW: i32 = 5;
pub const HWND_TOP: HWND = 0;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const WM_SETFONT: UINT = 0x0030;

pub const WC_TREEVIEWW: &str = "SysTreeView32";
pub const WC_EDITW: &str = "Edit";
pub const WC_BUTTONW: &str = "Button";
pub const WC_STATICW: &str = "Static";
