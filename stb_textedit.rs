#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// This is a slightly modified version of stb_textedit.h 1.14.
// Those changes would need to be pushed into nothings/stb:
// - Fix in stb_textedit_discard_redo (see https://github.com/nothings/stb/issues/321)
// - Fix in stb_textedit_find_charpos to handle last line (see https://github.com/ocornut/imgui/issues/6000 + #6783)
// - Added name to struct or it may be forward declared in our code.
// - Added UTF-8 support (see https://github.com/nothings/stb/issues/188 + https://github.com/ocornut/imgui/pull/7925)
// Grep for [DEAR IMGUI] to find the changes.
// - Also renamed macros used or defined outside of IMSTB_TEXTEDIT_IMPLEMENTATION block from STB_TEXTEDIT_* to IMSTB_TEXTEDIT_*

// stb_textedit.h - v1.14  - public domain - Sean Barrett
// Development of this library was sponsored by RAD Game Tools
//
// This C header file implements the guts of a multi-line text-editing
// widget; you implement display, word-wrapping, and low-level string
// insertion/deletion, and stb_textedit will map user inputs into
// insertions & deletions, plus updates to the cursor position,
// selection state, and undo state.
//
// It is intended for use in games and other systems that need to build
// their own custom widgets and which do not have heavy text-editing
// requirements (this library is not recommended for use for editing large
// texts, as its performance does not scale and it has limited undo).
//
// Non-trivial behaviors are modelled after Windows text controls.
//
//
// LICENSE
//
// See end of file for license information.
//
//
// DEPENDENCIES
//
// Uses the C runtime function 'memmove', which you can override
// by defining IMSTB_TEXTEDIT_memmove before the implementation.
// Uses no other functions. Performs no runtime allocations.
//
//
// VERSION HISTORY
//
//   1.14 (2021-07-11) page up/down, various fixes
//   1.13 (2019-02-07) fix bug in undo size management
//   1.12 (2018-01-29) user can change STB_TEXTEDIT_KEYTYPE, fix redo to avoid crash
//   1.11 (2017-03-03) fix HOME on last line, dragging off single-line textfield
//   1.10 (2016-10-25) suppress warnings about casting away const with -Wcast-qual
//   1.9  (2016-08-27) customizable move-by-word
//   1.8  (2016-04-02) better keyboard handling when mouse button is down
//   1.7  (2015-09-13) change y range handling in case baseline is non-0
//   1.6  (2015-04-15) allow STB_TEXTEDIT_memmove
//   1.5  (2014-09-10) add support for secondary keys for OS X
//   1.4  (2014-08-17) fix signed/unsigned warnings
//   1.3  (2014-06-19) fix mouse clicking to round to nearest char boundary
//   1.2  (2014-05-27) fix some RAD types that had crept into the new code
//   1.1  (2013-12-15) move-by-word (requires STB_TEXTEDIT_IS_SPACE )
//   1.0  (2012-07-26) improve documentation, initial public release
//   0.3  (2012-02-24) bugfixes, single-line mode; insert mode
//   0.2  (2011-11-28) fixes to undo/redo
//   0.1  (2010-07-08) initial version
//
// ADDITIONAL CONTRIBUTORS
//
//   Ulf Winklemann: move-by-word in 1.1
//   Fabian Giesen: secondary key inputs in 1.5
//   Martins Mozeiko: STB_TEXTEDIT_memmove in 1.6
//   Louis Schnellbach: page up/down in 1.14
//
//   Bugfixes:
//      Scott Graham
//      Daniel Keller
//      Omar Cornut
//      Dan Thompson
//
// USAGE
//
// This file behaves differently depending on what symbols you define
// before including it.
//
//
// Header-file mode:
//
//   If you do not define STB_TEXTEDIT_IMPLEMENTATION before including this,
//   it will operate in "header file" mode. In this mode, it declares a
//   single public symbol, STB_TexteditState, which encapsulates the current
//   state of a text widget (except for the string, which you will store
//   separately).
//
//   To compile in this mode, you must define STB_TEXTEDIT_CHARTYPE to a
//   primitive type that defines a single character (e.g. char, wchar_t, etc).
//
//   To save space or increase undo-ability, you can optionally define the
//   following things that are used by the undo system:
//
//      STB_TEXTEDIT_POSITIONTYPE         small int type encoding a valid cursor position
//      STB_TEXTEDIT_UNDOSTATECOUNT       the number of undo states to allow
//      STB_TEXTEDIT_UNDOCHARCOUNT        the number of characters to store in the undo buffer
//
//   If you don't define these, they are set to permissive types and
//   moderate sizes. The undo system does no memory allocations, so
//   it grows STB_TexteditState by the worst-case storage which is (in bytes):
//
//        [4 + 3 * sizeof(STB_TEXTEDIT_POSITIONTYPE)] * STB_TEXTEDIT_UNDOSTATECOUNT
//      +          sizeof(STB_TEXTEDIT_CHARTYPE)      * STB_TEXTEDIT_UNDOCHARCOUNT
//
//
// Implementation mode:
//
//   If you define STB_TEXTEDIT_IMPLEMENTATION before including this, it
//   will compile the implementation of the text edit widget, depending
//   on a large number of symbols which must be defined before the include.
//
//   The implementation is defined only as static functions. You will then
//   need to provide your own APIs in the same file which will access the
//   static functions.
//
//   The basic concept is that you provide a "string" object which
//   behaves like an array of characters. stb_textedit uses indices to
//   refer to positions in the string, implicitly representing positions
//   in the displayed textedit. This is true for both plain text and
//   rich text; even with rich text stb_truetype interacts with your
//   code as if there was an array of all the displayed characters.
//
// Symbols that must be the same in header-file and implementation mode:
//
//     STB_TEXTEDIT_CHARTYPE             the character type
//     STB_TEXTEDIT_POSITIONTYPE         small type that is a valid cursor position
//     STB_TEXTEDIT_UNDOSTATECOUNT       the number of undo states to allow
//     STB_TEXTEDIT_UNDOCHARCOUNT        the number of characters to store in the undo buffer
//
// Symbols you must define for implementation mode:
//
//    STB_TEXTEDIT_STRING               the type of object representing a string being edited,
//                                      typically this is a wrapper object with other data you need
//
//    STB_TEXTEDIT_STRINGLEN(obj)       the length of the string (ideally O(1))
//    STB_TEXTEDIT_LAYOUTROW(&r,obj,n)  returns the results of laying out a line of characters
//                                        starting from character #n (see discussion below)
//    STB_TEXTEDIT_GETWIDTH(obj,n,i)    returns the pixel delta from the xpos of the i'th character
//                                        to the xpos of the i+1'th char for a line of characters
//                                        starting at character #n (i.e. accounts for kerning
//                                        with previous char)
//    STB_TEXTEDIT_KEYTOTEXT(k)         maps a keyboard input to an insertable character
//                                        (return type is int, -1 means not valid to insert)
//                                        (not supported if you want to use UTF-8, see below)
//    STB_TEXTEDIT_GETCHAR(obj,i)       returns the i'th character of obj, 0-based
//    STB_TEXTEDIT_NEWLINE              the character returned by _GETCHAR() we recognize
//                                        as manually wordwrapping for end-of-line positioning
//
//    STB_TEXTEDIT_DELETECHARS(obj,i,n)      delete n characters starting at i
//    STB_TEXTEDIT_INSERTCHARS(obj,i,c*,n)   insert n characters at i (pointed to by STB_TEXTEDIT_CHARTYPE*)
//
//    STB_TEXTEDIT_K_SHIFT       a power of two that is or'd in to a keyboard input to represent the shift key
//
//    STB_TEXTEDIT_K_LEFT        keyboard input to move cursor left
//    STB_TEXTEDIT_K_RIGHT       keyboard input to move cursor right
//    STB_TEXTEDIT_K_UP          keyboard input to move cursor up
//    STB_TEXTEDIT_K_DOWN        keyboard input to move cursor down
//    STB_TEXTEDIT_K_PGUP        keyboard input to move cursor up a page
//    STB_TEXTEDIT_K_PGDOWN      keyboard input to move cursor down a page
//    STB_TEXTEDIT_K_LINESTART   keyboard input to move cursor to start of line  // e.g. HOME
//    STB_TEXTEDIT_K_LINEEND     keyboard input to move cursor to end of line    // e.g. END
//    STB_TEXTEDIT_K_TEXTSTART   keyboard input to move cursor to start of text  // e.g. ctrl-HOME
//    STB_TEXTEDIT_K_TEXTEND     keyboard input to move cursor to end of text    // e.g. ctrl-END
//    STB_TEXTEDIT_K_DELETE      keyboard input to delete selection or character under cursor
//    STB_TEXTEDIT_K_BACKSPACE   keyboard input to delete selection or character left of cursor
//    STB_TEXTEDIT_K_UNDO        keyboard input to perform undo
//    STB_TEXTEDIT_K_REDO        keyboard input to perform redo
//
// Optional:
//    STB_TEXTEDIT_K_INSERT              keyboard input to toggle insert mode
//    STB_TEXTEDIT_IS_SPACE(ch)          true if character is whitespace (e.g. 'isspace'),
//                                          required for default WORDLEFT/WORDRIGHT handlers
//    STB_TEXTEDIT_MOVEWORDLEFT(obj,i)   custom handler for WORDLEFT, returns index to move cursor to
//    STB_TEXTEDIT_MOVEWORDRIGHT(obj,i)  custom handler for WORDRIGHT, returns index to move cursor to
//    STB_TEXTEDIT_K_WORDLEFT            keyboard input to move cursor left one word // e.g. ctrl-LEFT
//    STB_TEXTEDIT_K_WORDRIGHT           keyboard input to move cursor right one word // e.g. ctrl-RIGHT
//    STB_TEXTEDIT_K_LINESTART2          secondary keyboard input to move cursor to start of line
//    STB_TEXTEDIT_K_LINEEND2            secondary keyboard input to move cursor to end of line
//    STB_TEXTEDIT_K_TEXTSTART2          secondary keyboard input to move cursor to start of text
//    STB_TEXTEDIT_K_TEXTEND2            secondary keyboard input to move cursor to end of text
//
// To support UTF-8:
//
//    STB_TEXTEDIT_GETPREVCHARINDEX      returns index of previous character
//    STB_TEXTEDIT_GETNEXTCHARINDEX      returns index of next character
//    Do NOT define STB_TEXTEDIT_KEYTOTEXT.
//    Instead, call stb_textedit_text() directly for text contents.
//
// Keyboard input must be encoded as a single integer value; e.g. a character code
// and some bitflags that represent shift states. to simplify the interface, SHIFT must
// be a bitflag, so we can test the shifted state of cursor movements to allow selection,
// i.e. (STB_TEXTEDIT_K_RIGHT|STB_TEXTEDIT_K_SHIFT) should be shifted right-arrow.
//
// You can encode other things, such as CONTROL or ALT, in additional bits, and
// then test for their presence in e.g. STB_TEXTEDIT_K_WORDLEFT. For example,
// my Windows implementations add an additional CONTROL bit, and an additional KEYDOWN
// bit. Then all of the STB_TEXTEDIT_K_ values bitwise-or in the KEYDOWN bit,
// and I pass both WM_KEYDOWN and WM_CHAR events to the "key" function in the
// API below. The control keys will only match WM_KEYDOWN events because of the
// keydown bit I add, and STB_TEXTEDIT_KEYTOTEXT only tests for the KEYDOWN
// bit so it only decodes WM_CHAR events.
//
// STB_TEXTEDIT_LAYOUTROW returns information about the shape of one displayed
// row of characters assuming they start on the i'th character--the width and
// the height and the number of characters consumed. This allows this library
// to traverse the entire layout incrementally. You need to compute word-wrapping
// here.
//
// Each textfield keeps its own insert mode state, which is not how normal
// applications work. To keep an app-wide insert mode, update/copy the
// "insert_mode" field of STB_TexteditState before/after calling API functions.
//
// API
//
//    void stb_textedit_initialize_state(STB_TexteditState *state, int is_single_line)
//
//    void stb_textedit_click(STB_TEXTEDIT_STRING *str, STB_TexteditState *state, float x, float y)
//    void stb_textedit_drag(STB_TEXTEDIT_STRING *str, STB_TexteditState *state, float x, float y)
//    int  stb_textedit_cut(STB_TEXTEDIT_STRING *str, STB_TexteditState *state)
//    int  stb_textedit_paste(STB_TEXTEDIT_STRING *str, STB_TexteditState *state, STB_TEXTEDIT_CHARTYPE *text, int len)
//    void stb_textedit_key(STB_TEXTEDIT_STRING *str, STB_TexteditState *state, STB_TEXEDIT_KEYTYPE key)
//    void stb_textedit_text(STB_TEXTEDIT_STRING *str, STB_TexteditState *state, STB_TEXTEDIT_CHARTYPE *text, int text_len)
//
//    Each of these functions potentially updates the string and updates the
//    state.
//
//      initialize_state:
//          set the textedit state to a known good default state when initially
//          constructing the textedit.
//
//      click:
//          call this with the mouse x,y on a mouse down; it will update the cursor
//          and reset the selection start/end to the cursor point. the x,y must
//          be relative to the text widget, with (0,0) being the top left.
//
//      drag:
//          call this with the mouse x,y on a mouse drag/up; it will update the
//          cursor and the selection end point
//
//      cut:
//          call this to delete the current selection; returns true if there was
//          one. you should FIRST copy the current selection to the system paste buffer.
//          (To copy, just copy the current selection out of the string yourself.)
//
//      paste:
//          call this to paste text at the current cursor point or over the current
//          selection if there is one.
//
//      key:
//          call this for keyboard inputs sent to the textfield. you can use it
//          for "key down" events or for "translated" key events. if you need to
//          do both (as in Win32), or distinguish Unicode characters from control
//          inputs, set a high bit to distinguish the two; then you can define the
//          various definitions like STB_TEXTEDIT_K_LEFT have the is-key-event bit
//          set, and make STB_TEXTEDIT_KEYTOCHAR check that the is-key-event bit is
//          clear. STB_TEXTEDIT_KEYTYPE defaults to int, but you can #define it to
//          anything other type you want before including.
//          if the STB_TEXTEDIT_KEYTOTEXT function is defined, selected keys are
//          transformed into text and stb_textedit_text() is automatically called.
//
//      text: (added 2025)
//          call this to directly send text input the textfield, which is required
//          for UTF-8 support, because stb_textedit_key() + STB_TEXTEDIT_KEYTOTEXT()
//          cannot infer text length.
//
//
//   When rendering, you can read the cursor position and selection state from
//   the STB_TexteditState.
//
//
// Notes:
//
// This is designed to be usable in IMGUI, so it allows for the possibility of
// running in an IMGUI that has NOT cached the multi-line layout. For this
// reason, it provides an interface that is compatible with computing the
// layout incrementally--we try to make sure we make as few passes through
// as possible. (For example, to locate the mouse pointer in the text, we
// could define functions that return the X and Y positions of characters
// and binary search Y and then X, but if we're doing dynamic layout this
// will run the layout algorithm many times, so instead we manually search
// forward in one pass. Similar logic applies to e.g. up-arrow and
// down-arrow movement.)
//
// If it's run in a widget that *has* cached the layout, then this is less
// efficient, but it's not horrible on modern computers. But you wouldn't
// want to edit million-line files with it.

type int = i32;
type short = i16;
type float = f32;
type unsigned_char = u8;

macro_rules! stb_textedit_k {
    ($key:ident $val:literal) => {
        pub const $key: STB_TEXTEDIT_KEYTYPE = $val;
    };
}

stb_textedit_k!(STB_TEXTEDIT_K_LEFT         0x200000); // keyboard input to move cursor left
stb_textedit_k!(STB_TEXTEDIT_K_RIGHT        0x200001); // keyboard input to move cursor right
stb_textedit_k!(STB_TEXTEDIT_K_UP           0x200002); // keyboard input to move cursor up
stb_textedit_k!(STB_TEXTEDIT_K_DOWN         0x200003); // keyboard input to move cursor down
stb_textedit_k!(STB_TEXTEDIT_K_LINESTART    0x200004); // keyboard input to move cursor to start of line
stb_textedit_k!(STB_TEXTEDIT_K_LINEEND      0x200005); // keyboard input to move cursor to end of line
stb_textedit_k!(STB_TEXTEDIT_K_TEXTSTART    0x200006); // keyboard input to move cursor to start of text
stb_textedit_k!(STB_TEXTEDIT_K_TEXTEND      0x200007); // keyboard input to move cursor to end of text
stb_textedit_k!(STB_TEXTEDIT_K_DELETE       0x200008); // keyboard input to delete selection or character under cursor
stb_textedit_k!(STB_TEXTEDIT_K_BACKSPACE    0x200009); // keyboard input to delete selection or character left of cursor
stb_textedit_k!(STB_TEXTEDIT_K_UNDO         0x20000A); // keyboard input to perform undo
stb_textedit_k!(STB_TEXTEDIT_K_REDO         0x20000B); // keyboard input to perform redo
stb_textedit_k!(STB_TEXTEDIT_K_WORDLEFT     0x20000C); // keyboard input to move cursor left one word
stb_textedit_k!(STB_TEXTEDIT_K_WORDRIGHT    0x20000D); // keyboard input to move cursor right one word
stb_textedit_k!(STB_TEXTEDIT_K_PGUP         0x20000E); // keyboard input to move cursor up a page
stb_textedit_k!(STB_TEXTEDIT_K_PGDOWN       0x20000F); // keyboard input to move cursor down a page
stb_textedit_k!(STB_TEXTEDIT_K_SHIFT        0x400000);

pub type STB_TEXTEDIT_STRING = String;

pub fn STB_TEXTEDIT_STRINGLEN(str: &STB_TEXTEDIT_STRING) -> int {
    str.len() as int
}

pub fn STB_TEXTEDIT_LAYOUTROW(r: &mut StbTexteditRow, str: &STB_TEXTEDIT_STRING, i: int) {}

pub fn STB_TEXTEDIT_GETCHAR(obj: &STB_TEXTEDIT_STRING, i: int) -> STB_TEXTEDIT_CHARTYPE {
    todo!()
}

pub fn STB_TEXTEDIT_DELETECHARS(
    obj: &mut STB_TEXTEDIT_STRING,
    i: int,
    len: int,
) -> STB_TEXTEDIT_CHARTYPE {
    todo!()
}

pub fn STB_TEXTEDIT_GETWIDTH(obj: &STB_TEXTEDIT_STRING, n: int, i: int) -> float {
    todo!()
}

pub fn STB_TEXTEDIT_INSERTCHARS(
    obj: &mut STB_TEXTEDIT_STRING,
    i: int,
    c: &[STB_TEXTEDIT_CHARTYPE],
) -> bool {
    todo!()
}

pub fn STB_TEXTEDIT_KEYTOTEXT(k: STB_TEXTEDIT_KEYTYPE) -> STB_TEXTEDIT_CHARTYPE {
    todo!()
}

pub const STB_TEXTEDIT_NEWLINE: STB_TEXTEDIT_CHARTYPE = '\n' as STB_TEXTEDIT_CHARTYPE;

////////////////////////////////////////////////////////////////////////
//
//     STB_TexteditState
//
// Definition of STB_TexteditState which you should store
// per-textfield; it includes cursor position, selection state,
// and undo state.
//

macro_rules! STB_TEXTEDIT_UNDOSTATECOUNT {
    () => {
        99
    };
}
macro_rules! STB_TEXTEDIT_UNDOCHARCOUNT {
    () => {
        999
    };
}

pub type STB_TEXTEDIT_CHARTYPE = int;
pub type STB_TEXTEDIT_POSITIONTYPE = int;
pub type STB_TEXTEDIT_KEYTYPE = int;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StbUndoRecord {
    // private data
    pub location: STB_TEXTEDIT_POSITIONTYPE,
    pub insert_length: STB_TEXTEDIT_POSITIONTYPE,
    pub delete_length: STB_TEXTEDIT_POSITIONTYPE,
    pub char_storage: int,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StbUndoState {
    // private data
    pub undo_rec: [StbUndoRecord; STB_TEXTEDIT_UNDOSTATECOUNT!()],
    pub undo_char: [STB_TEXTEDIT_CHARTYPE; STB_TEXTEDIT_UNDOCHARCOUNT!()],
    pub undo_point: short,
    pub redo_point: short,
    pub undo_char_point: int,
    pub redo_char_point: int,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct STB_TexteditState {
    /////////////////////
    //
    // public data
    //
    /// position of the text cursor within the string
    pub cursor: int,

    /// selection start point
    ///
    /// selection start and end point in characters; if equal, no selection.
    /// note that start may be less than or greater than end (e.g. when
    /// dragging the mouse, start is where the initial click was, and you
    /// can drag in either direction)
    pub select_start: int,
    /// selection end point
    ///
    /// see [`Self::select_start`]
    pub select_end: int,

    /// each textfield keeps its own insert mode state. to keep an app-wide
    /// insert mode, copy this value in/out of the app state
    pub insert_mode: unsigned_char,

    /// page size in number of row.
    /// this value MUST be set to >0 for pageup or pagedown in multilines documents.
    pub row_count_per_page: int,

    /////////////////////
    //
    // private data
    //
    /// not implemented yet
    pub cursor_at_end_of_line: unsigned_char,
    pub initialized: unsigned_char,
    pub has_preferred_x: unsigned_char,
    pub single_line: unsigned_char,
    pub padding1: unsigned_char,
    pub padding2: unsigned_char,
    pub padding3: unsigned_char,
    /// this determines where the cursor up/down tries to seek to along x
    pub preferred_x: float,
    pub undostate: StbUndoState,
}

////////////////////////////////////////////////////////////////////////
//
//     StbTexteditRow
//
// Result of layout query, used by stb_textedit to determine where
// the text in each row is.

/// result of layout query
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StbTexteditRow {
    /// starting x location, end x location (allows for align=right, etc)
    pub x0: float,
    pub x1: float,
    /// position of baseline relative to previous row's baseline
    pub baseline_y_delta: float,
    /// height of row above baseline
    pub ymin: float,
    /// height of row below baseline
    pub ymax: float,
    pub num_chars: int,
}

pub const fn StbTexteditRow() -> StbTexteditRow {
    StbTexteditRow {
        x0: 0.0,
        x1: 0.0,
        baseline_y_delta: 0.0,
        ymin: 0.0,
        ymax: 0.0,
        num_chars: 0,
    }
}

#[inline]
fn stb_textedit_memmove<T: Copy>(slice: &mut [T], dest_idx: usize, src_idx: usize, count: usize) {
    if dest_idx == src_idx || count == 0 {
        return;
    }

    // Safe version of memmove logic
    if dest_idx < src_idx {
        for i in 0..count {
            slice[dest_idx + i] = slice[src_idx + i];
        }
    } else {
        for i in (0..count).rev() {
            slice[dest_idx + i] = slice[src_idx + i];
        }
    }
}

macro_rules! STB_TEXTEDIT_memmove {
    ($slice:expr, $dest:expr, $src:expr, $count:expr) => {
        stb_textedit_memmove($slice, ($dest) as usize, ($src) as usize, ($count) as usize)
    };
}

macro_rules! STB_TEXTEDIT_GETPREVCHARINDEX {
    ($OBJ:expr, $IDX:expr) => {
        (($IDX) - 1)
    };
}

macro_rules! STB_TEXTEDIT_GETNEXTCHARINDEX {
    ($OBJ:expr, $IDX:expr) => {
        (($IDX) + 1)
    };
}

macro_rules! c_for {
    ($init:stmt; $cond:expr; $incr:stmt; $body:block) => {{
        $init
        while $cond {
            $body
            $incr
        }
    }};
}

/////////////////////////////////////////////////////////////////////////////
//
//      Mouse input handling
//

/// traverse the layout to locate the nearest character to a display position
pub fn stb_text_locate_coord(
    str: &STB_TEXTEDIT_STRING,
    x: float,
    y: float,
    out_side_on_line: &mut int,
) -> int {
    let mut r = StbTexteditRow();
    let n = STB_TEXTEDIT_STRINGLEN(str);
    let mut base_y = 0.0;
    let mut prev_x;
    let mut i = 0;

    *out_side_on_line = 0;

    // search rows to find one that straddles 'y'
    while i < n {
        STB_TEXTEDIT_LAYOUTROW(&mut r, str, i);
        if r.num_chars <= 0 {
            return n;
        }

        if i == 0 && y < base_y + r.ymin {
            return 0;
        }

        if y < base_y + r.ymax {
            break;
        }

        i += r.num_chars;
        base_y += r.baseline_y_delta;
    }

    // below all text, return 'after' last character
    if i >= n {
        *out_side_on_line = 1;
        return n;
    }

    // check if it's before the beginning of the line
    if x < r.x0 {
        return i;
    }

    // check if it's before the end of the line
    if x < r.x1 {
        // search characters in row for one that straddles 'x'
        prev_x = r.x0;
        c_for!(let mut k=0; k < r.num_chars; k = STB_TEXTEDIT_GETNEXTCHARINDEX!(str, i + k) - i; {
           let w = STB_TEXTEDIT_GETWIDTH(str, i, k);
           if x < prev_x+w {
              *out_side_on_line = if k == 0 { 0 } else { 1 };
              if x < prev_x+w/2.0 {
                 return k+i;
              } else {
                 return STB_TEXTEDIT_GETNEXTCHARINDEX!(str, i + k);
              }
           }
           prev_x += w;
        });
        // shouldn't happen, but if it does, fall through to end-of-line case
    }

    // if the last character is a newline, return that. otherwise return 'after' the last character
    *out_side_on_line = 1;
    if STB_TEXTEDIT_GETCHAR(str, i + r.num_chars - 1) == STB_TEXTEDIT_NEWLINE {
        i + r.num_chars - 1
    } else {
        i + r.num_chars
    }
}

/// API click: on mouse down, move the cursor to the clicked location, and reset the selection
pub fn stb_textedit_click(
    str: &STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    x: float,
    mut y: float,
) {
    // In single-line mode, just always make y = 0. This lets the drag keep working if the mouse
    // goes off the top or bottom of the text
    let mut side_on_line = 0;
    if state.single_line != 0 {
        let mut r = StbTexteditRow();
        STB_TEXTEDIT_LAYOUTROW(&mut r, str, 0);
        y = r.ymin;
    }

    state.cursor = stb_text_locate_coord(str, x, y, &mut side_on_line);
    state.select_start = state.cursor;
    state.select_end = state.cursor;
    state.has_preferred_x = 0;

    // TODO
    // str.LastMoveDirectionLR = (ImS8)(side_on_line ? ImGuiDir_Right : ImGuiDir_Left);
}

/// API drag: on mouse drag, move the cursor and selection endpoint to the clicked location
pub fn stb_textedit_drag(
    str: &STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    x: float,
    mut y: float,
) {
    let mut p = 0;
    let mut side_on_line = 0;

    // In single-line mode, just always make y = 0. This lets the drag keep working if the mouse
    // goes off the top or bottom of the text
    if state.single_line != 0 {
        let mut r = StbTexteditRow();
        STB_TEXTEDIT_LAYOUTROW(&mut r, str, 0);
        y = r.ymin;
    }

    if state.select_start == state.select_end {
        state.select_start = state.cursor;
    }

    p = stb_text_locate_coord(str, x, y, &mut side_on_line);
    state.cursor = p;
    state.select_end = p;

    // TODO
    // str.LastMoveDirectionLR = (ImS8)(side_on_line ? ImGuiDir_Right : ImGuiDir_Left);
}

/////////////////////////////////////////////////////////////////////////////
//
//      Keyboard input handling
//

// forward declarations

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StbFindState {
    /// x position of n'th character
    pub x: float,
    /// y position of n'th character
    pub y: float,
    /// height of line
    pub height: float,
    /// first char of row
    pub first_char: int,
    /// length of row
    pub length: int,
    /// first char of previous row
    pub prev_first: int,
}

pub const fn StbFindState() -> StbFindState {
    StbFindState {
        x: 0.0,
        y: 0.0,
        height: 0.0,
        first_char: 0,
        length: 0,
        prev_first: 0,
    }
}

// find the x/y location of a character, and remember info about the previous row in
// case we get a move-up event (for page up, we'll have to rescan)
pub fn stb_textedit_find_charpos(
    find: &mut StbFindState,
    str: &STB_TEXTEDIT_STRING,
    n: int,
    single_line: int,
) {
    let mut r = StbTexteditRow();
    let mut prev_start = 0;
    let z = STB_TEXTEDIT_STRINGLEN(str);
    let mut i = 0;

    if n == z && single_line != 0 {
        // special case if it's at the end (may not be needed?)
        STB_TEXTEDIT_LAYOUTROW(&mut r, str, 0);
        find.y = 0.0;
        find.first_char = 0;
        find.length = z;
        find.height = r.ymax - r.ymin;
        find.x = r.x1;
        return;
    }

    // search rows to find the one that straddles character n
    find.y = 0.0;

    loop {
        STB_TEXTEDIT_LAYOUTROW(&mut r, str, i);
        if n < i + r.num_chars {
            break;
        }
        // if (str.LastMoveDirectionLR == ImGuiDir_Right && str.Stb.cursor > 0 && str.Stb.cursor == i + r.num_chars && STB_TEXTEDIT_GETCHAR(str, i + r.num_chars - 1) != STB_TEXTEDIT_NEWLINE) // [DEAR IMGUI] Wrapping point handling
        //    break;
        if i + r.num_chars == z && z > 0 && STB_TEXTEDIT_GETCHAR(str, z - 1) != STB_TEXTEDIT_NEWLINE
        {
            // [DEAR IMGUI] special handling for last line
            break; // [DEAR IMGUI]
        }
        prev_start = i;
        i += r.num_chars;
        find.y += r.baseline_y_delta;
        if i == z
        // [DEAR IMGUI]
        {
            r.num_chars = 0; // [DEAR IMGUI]
            break; // [DEAR IMGUI]
        }
    }

    find.first_char = i;
    let first = i;
    find.length = r.num_chars;
    find.height = r.ymax - r.ymin;
    find.prev_first = prev_start;

    // now scan to find xpos
    find.x = r.x0;
    c_for!(i=0; first+i < n; i = STB_TEXTEDIT_GETNEXTCHARINDEX!(str, first + i) - first; {
       find.x += STB_TEXTEDIT_GETWIDTH(str, first, i);
    });
}

macro_rules! STB_TEXT_HAS_SELECTION {
    ($s:expr) => {
        ($s).select_start != ($s).select_end
    };
}

// make the selection/cursor state valid if client altered the string
pub fn stb_textedit_clamp(str: &STB_TEXTEDIT_STRING, state: &mut STB_TexteditState) {
    let n = STB_TEXTEDIT_STRINGLEN(str);
    if STB_TEXT_HAS_SELECTION!(state) {
        if state.select_start > n {
            state.select_start = n;
        }
        if state.select_end > n {
            state.select_end = n;
        }
        // if clamping forced them to be equal, move the cursor to match
        if state.select_start == state.select_end {
            state.cursor = state.select_start;
        }
    }
    if state.cursor > n {
        state.cursor = n;
    }
}

/// delete characters while updating undo
pub fn stb_textedit_delete(
    str: &mut STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    location: int,
    len: int,
) {
    stb_text_makeundo_delete(str, state, location, len);
    STB_TEXTEDIT_DELETECHARS(str, location, len);
    state.has_preferred_x = 0;
}

// delete the section
pub fn stb_textedit_delete_selection(str: &mut STB_TEXTEDIT_STRING, state: &mut STB_TexteditState) {
    stb_textedit_clamp(str, state);
    if STB_TEXT_HAS_SELECTION!(state) {
        if state.select_start < state.select_end {
            stb_textedit_delete(
                str,
                state,
                state.select_start,
                state.select_end - state.select_start,
            );
            state.select_end = state.select_start;
            state.cursor = state.select_start;
        } else {
            stb_textedit_delete(
                str,
                state,
                state.select_end,
                state.select_start - state.select_end,
            );
            state.select_start = state.select_end;
            state.cursor = state.select_end;
        }
        state.has_preferred_x = 0;
    }
}

// canoncialize the selection so start <= end
pub fn stb_textedit_sortselection(state: &mut STB_TexteditState) {
    if state.select_end < state.select_start {
        std::mem::swap(&mut state.select_end, &mut state.select_start);
    }
}

// move cursor to first character of selection
pub fn stb_textedit_move_to_first(state: &mut STB_TexteditState) {
    if STB_TEXT_HAS_SELECTION!(state) {
        stb_textedit_sortselection(state);
        state.cursor = state.select_start;
        state.select_end = state.select_start;
        state.has_preferred_x = 0;
    }
}

// move cursor to last character of selection
pub fn stb_textedit_move_to_last(str: &STB_TEXTEDIT_STRING, state: &mut STB_TexteditState) {
    if STB_TEXT_HAS_SELECTION!(state) {
        stb_textedit_sortselection(state);
        stb_textedit_clamp(str, state);
        state.cursor = state.select_end;
        state.select_start = state.select_end;
        state.has_preferred_x = 0;
    }
}

pub fn stb_textedit_move_line_start(
    str: &STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    mut cursor: int,
) -> int {
    if state.single_line != 0 {
        return 0;
    }
    while cursor > 0 {
        let prev = STB_TEXTEDIT_GETPREVCHARINDEX!(str, cursor);
        if STB_TEXTEDIT_GETCHAR(str, prev) == STB_TEXTEDIT_NEWLINE {
            break;
        }
        cursor = prev;
    }
    cursor
}

pub fn STB_TEXTEDIT_MOVELINESTART(
    str: &STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    cursor: int,
) -> int {
    stb_textedit_move_line_start(str, state, cursor)
}

pub fn stb_textedit_move_line_end(
    str: &STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    mut cursor: int,
) -> int {
    let n = STB_TEXTEDIT_STRINGLEN(str);
    if state.single_line != 0 {
        return n;
    }
    while cursor < n && STB_TEXTEDIT_GETCHAR(str, cursor) != STB_TEXTEDIT_NEWLINE {
        cursor += 1;
    }
    cursor
}

pub fn STB_TEXTEDIT_MOVELINEEND(
    str: &STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    cursor: int,
) -> int {
    stb_textedit_move_line_end(str, state, cursor)
}

// pub fn is_word_boundary(str: &STB_TEXTEDIT_STRING, idx: int)
// {
//    return idx > 0 ? (STB_TEXTEDIT_IS_SPACE( STB_TEXTEDIT_GETCHAR(str,idx-1) ) && !STB_TEXTEDIT_IS_SPACE( STB_TEXTEDIT_GETCHAR(str, idx) ) ) : 1;
// }

// pub fn stb_textedit_move_to_word_previous( str: &STB_TEXTEDIT_STRING, c: int ) -> int
// {
//    c = STB_TEXTEDIT_GETPREVCHARINDEX!( str, c ); // always move at least one character
//    while (c >= 0 && !is_word_boundary(str, c)) {
//       c = STB_TEXTEDIT_GETPREVCHARINDEX!(str, c);
//    }

//    if( c < 0 ) {
//       c = 0;
//    }

//    return c;

// }

// pub fn stb_textedit_move_to_word_next( str: &STB_TEXTEDIT_STRING, int c ) -> int
// {
//    let len = STB_TEXTEDIT_STRINGLEN(str);
//    c = STB_TEXTEDIT_GETNEXTCHARINDEX!(str, c); // always move at least one character
//    while( c < len && !is_word_boundary( str, c ) ) {
//       c = STB_TEXTEDIT_GETNEXTCHARINDEX!(str, c);
//    }

//    if( c > len ) {
//       c = len;
//    }

//    return c;
// }

// update selection and cursor to match each other
pub fn stb_textedit_prep_selection_at_cursor(state: &mut STB_TexteditState) {
    if !STB_TEXT_HAS_SELECTION!(state) {
        state.select_start = state.cursor;
        state.select_end = state.cursor;
    } else {
        state.cursor = state.select_end;
    }
}

// API cut: delete selection
pub fn stb_textedit_cut(str: &mut STB_TEXTEDIT_STRING, state: &mut STB_TexteditState) -> int {
    if STB_TEXT_HAS_SELECTION!(state) {
        stb_textedit_delete_selection(str, state); // implicitly clamps
        state.has_preferred_x = 0;
        return 1;
    }
    0
}

// API paste: replace existing selection with passed-in text
// TODO: slice
pub fn stb_textedit_paste_internal(
    str: &mut STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    text: &[STB_TEXTEDIT_CHARTYPE],
) -> int {
    // if there's a selection, the paste should delete it
    let len = text.len() as int;
    stb_textedit_clamp(str, state);
    stb_textedit_delete_selection(str, state);
    // try to insert the characters
    if STB_TEXTEDIT_INSERTCHARS(str, state.cursor, text) {
        stb_text_makeundo_insert(state, state.cursor, len);
        state.cursor += len;
        state.has_preferred_x = 0;
        return 1;
    }
    // note: paste failure will leave deleted selection, may be restored with an undo (see https://github.com/nothings/stb/issues/734 for details)
    0
}

// API key: process text input
// [DEAR IMGUI] Added stb_textedit_text(), extracted out and called by stb_textedit_key() for backward compatibility.
pub fn stb_textedit_text(
    str: &mut STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    text: &[STB_TEXTEDIT_CHARTYPE],
) {
    let text_len = text.len() as int;
    // can't add newline in single-line mode
    if text[0] == STB_TEXTEDIT_NEWLINE && state.single_line != 0 {
        return;
    }

    if state.insert_mode != 0
        && !STB_TEXT_HAS_SELECTION!(state)
        && state.cursor < STB_TEXTEDIT_STRINGLEN(str)
    {
        stb_text_makeundo_replace(str, state, state.cursor, 1, 1);
        STB_TEXTEDIT_DELETECHARS(str, state.cursor, 1);
        if STB_TEXTEDIT_INSERTCHARS(str, state.cursor, text) {
            state.cursor += text_len;
            state.has_preferred_x = 0;
        }
    } else {
        stb_textedit_delete_selection(str, state); // implicitly clamps
        if STB_TEXTEDIT_INSERTCHARS(str, state.cursor, text) {
            stb_text_makeundo_insert(state, state.cursor, text_len);
            state.cursor += text_len;
            state.has_preferred_x = 0;
        }
    }
}

// API key: process a keyboard input
pub fn stb_textedit_key(
    str: &mut STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    mut key: STB_TEXTEDIT_KEYTYPE,
) {
    // STB_TEXTEDIT_K_INSERT => {
    //     state.insert_mode = !state.insert_mode;
    // }
    if key == STB_TEXTEDIT_K_UNDO {
        stb_text_undo(str, state);
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_REDO {
        stb_text_redo(str, state);
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_LEFT {
        // if currently there's a selection, move cursor to start of selection
        if STB_TEXT_HAS_SELECTION!(state) {
            stb_textedit_move_to_first(state);
        } else {
            if state.cursor > 0 {
                state.cursor = STB_TEXTEDIT_GETPREVCHARINDEX!(str, state.cursor);
            }
        }
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_RIGHT {
        // if currently there's a selection, move cursor to end of selection
        if STB_TEXT_HAS_SELECTION!(state) {
            stb_textedit_move_to_last(str, state);
        } else {
            state.cursor = STB_TEXTEDIT_GETNEXTCHARINDEX!(str, state.cursor);
        }
        stb_textedit_clamp(str, state);
        state.has_preferred_x = 0;
    } else if key == (STB_TEXTEDIT_K_LEFT | STB_TEXTEDIT_K_SHIFT) {
        stb_textedit_clamp(str, state);
        stb_textedit_prep_selection_at_cursor(state);
        // move selection left
        if state.select_end > 0 {
            state.select_end = STB_TEXTEDIT_GETPREVCHARINDEX!(str, state.select_end);
        }
        state.cursor = state.select_end;
        state.has_preferred_x = 0;
    } else if key == (STB_TEXTEDIT_K_RIGHT | STB_TEXTEDIT_K_SHIFT) {
        stb_textedit_prep_selection_at_cursor(state);
        // move selection right
        state.select_end = STB_TEXTEDIT_GETNEXTCHARINDEX!(str, state.select_end);
        stb_textedit_clamp(str, state);
        state.cursor = state.select_end;
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_DOWN
        || key == (STB_TEXTEDIT_K_DOWN | STB_TEXTEDIT_K_SHIFT)
        || key == STB_TEXTEDIT_K_PGDOWN
        || key == (STB_TEXTEDIT_K_PGDOWN | STB_TEXTEDIT_K_SHIFT)
    {
        let mut find = StbFindState();
        let mut row = StbTexteditRow();
        let mut i;
        let mut j;
        let sel = (key & STB_TEXTEDIT_K_SHIFT) != 0;
        // let is_page = (key & ~STB_TEXTEDIT_K_SHIFT) == STB_TEXTEDIT_K_PGDOWN;
        let is_page = (key & !STB_TEXTEDIT_K_SHIFT) == STB_TEXTEDIT_K_PGDOWN;
        let row_count = if is_page { state.row_count_per_page } else { 1 };

        if !is_page && state.single_line != 0 {
            // on windows, up&down in single-line behave like left&right
            key = STB_TEXTEDIT_K_RIGHT | (key & STB_TEXTEDIT_K_SHIFT);
            return stb_textedit_key(str, state, key);
            // goto retry;
        }

        if sel {
            stb_textedit_prep_selection_at_cursor(state);
        } else if STB_TEXT_HAS_SELECTION!(state) {
            stb_textedit_move_to_last(str, state);
        }

        // compute current position of cursor point
        stb_textedit_clamp(str, state);
        stb_textedit_find_charpos(&mut find, str, state.cursor, state.single_line as int);

        c_for!(j = 0; j < row_count; j+=1; {
            let mut x;
            let goal_x = if state.has_preferred_x != 0 { state.preferred_x } else { find.x };
            let start = find.first_char + find.length;

            if find.length == 0 {
                break;
            }

            // [DEAR IMGUI]
            // going down while being on the last line shouldn't bring us to that line end
            //if (STB_TEXTEDIT_GETCHAR(str, find.first_char + find.length - 1) != STB_TEXTEDIT_NEWLINE)
            //   break;

            // now find character position down a row
            state.cursor = start;
            STB_TEXTEDIT_LAYOUTROW(&mut row, str, state.cursor);
            x = row.x0;
            c_for!(i=0; i < row.num_chars; {}; {
                let dx = STB_TEXTEDIT_GETWIDTH(str, start, i);
                let next = STB_TEXTEDIT_GETNEXTCHARINDEX!(str, state.cursor);
                // #ifdef IMSTB_TEXTEDIT_GETWIDTH_NEWLINE
                // if (dx == IMSTB_TEXTEDIT_GETWIDTH_NEWLINE)
                //    break;
                // #endif
                x += dx;
                if x > goal_x {
                    break;
                }
                i += next - state.cursor;
                state.cursor = next;
            });
            stb_textedit_clamp(str, state);

            // if (state.cursor == find.first_char + find.length)
            //    str.LastMoveDirectionLR = ImGuiDir_Left;
            state.has_preferred_x = 1;
            state.preferred_x = goal_x;

            if sel {
                state.select_end = state.cursor;
            }

            // go to next line
            find.first_char = find.first_char + find.length;
            find.length = row.num_chars;
        });
    } else if key == STB_TEXTEDIT_K_UP
        || key == (STB_TEXTEDIT_K_UP | STB_TEXTEDIT_K_SHIFT)
        || key == STB_TEXTEDIT_K_PGUP
        || key == (STB_TEXTEDIT_K_PGUP | STB_TEXTEDIT_K_SHIFT)
    {
        let mut find = StbFindState();
        let mut row = StbTexteditRow();
        let mut i;
        let mut j;
        let mut prev_scan;
        let sel = (key & STB_TEXTEDIT_K_SHIFT) != 0;
        // let is_page = (key & ~STB_TEXTEDIT_K_SHIFT) == STB_TEXTEDIT_K_PGUP;
        let is_page = (key & !STB_TEXTEDIT_K_SHIFT) == STB_TEXTEDIT_K_PGUP;
        let row_count = if is_page { state.row_count_per_page } else { 1 };

        if !is_page && state.single_line != 0 {
            // on windows, up&down become left&right
            key = STB_TEXTEDIT_K_LEFT | (key & STB_TEXTEDIT_K_SHIFT);
            return stb_textedit_key(str, state, key);
        }

        if sel {
            stb_textedit_prep_selection_at_cursor(state);
        } else if STB_TEXT_HAS_SELECTION!(state) {
            stb_textedit_move_to_first(state)
        }

        // compute current position of cursor point
        stb_textedit_clamp(str, state);
        stb_textedit_find_charpos(&mut find, str, state.cursor, state.single_line as int);

        c_for!(j = 0; j < row_count; j += 1; {
            let mut x;
            let goal_x = if state.has_preferred_x != 0 { state.preferred_x } else { find.x };

            // can only go up if there's a previous row
            if find.prev_first == find.first_char {
                break;
            }

            // now find character position up a row
            state.cursor = find.prev_first;
            STB_TEXTEDIT_LAYOUTROW(&mut row, str, state.cursor);
            x = row.x0;
            c_for!(i=0; i < row.num_chars; {}; {
                let dx = STB_TEXTEDIT_GETWIDTH(str, find.prev_first, i);
                let next = STB_TEXTEDIT_GETNEXTCHARINDEX!(str, state.cursor);
                // #ifdef IMSTB_TEXTEDIT_GETWIDTH_NEWLINE
                // if (dx == IMSTB_TEXTEDIT_GETWIDTH_NEWLINE)
                //    break;
                // #endif
                x += dx;
                if x > goal_x {
                    break;
                }
                i += next - state.cursor;
                state.cursor = next;
            });
            stb_textedit_clamp(str, state);

            // if (state.cursor == find.first_char)
            //    str.LastMoveDirectionLR = ImGuiDir_Right;
            // else if (state.cursor == find.prev_first)
            //    str.LastMoveDirectionLR = ImGuiDir_Left;

            state.has_preferred_x = 1;
            state.preferred_x = goal_x;

            if sel {
                state.select_end = state.cursor;
            }

            // go to previous line
            // (we need to scan previous line the hard way. maybe we could expose this as a new API function?)
            prev_scan = if find.prev_first > 0 { find.prev_first - 1 } else { 0 };
            while prev_scan > 0
            {
                let prev = STB_TEXTEDIT_GETPREVCHARINDEX!(str, prev_scan);
                if STB_TEXTEDIT_GETCHAR(str, prev) == STB_TEXTEDIT_NEWLINE {
                    break;
                }
                prev_scan = prev;
            }
            find.first_char = find.prev_first;
            find.prev_first = STB_TEXTEDIT_MOVELINESTART(str, state, prev_scan);
        });
    } else if key == STB_TEXTEDIT_K_DELETE || key == (STB_TEXTEDIT_K_DELETE | STB_TEXTEDIT_K_SHIFT)
    {
        if STB_TEXT_HAS_SELECTION!(state) {
            stb_textedit_delete_selection(str, state);
        } else {
            let n = STB_TEXTEDIT_STRINGLEN(str);
            if state.cursor < n {
                stb_textedit_delete(
                    str,
                    state,
                    state.cursor,
                    STB_TEXTEDIT_GETNEXTCHARINDEX!(str, state.cursor) - state.cursor,
                );
            }
        }
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_BACKSPACE
        || key == (STB_TEXTEDIT_K_BACKSPACE | STB_TEXTEDIT_K_SHIFT)
    {
        if STB_TEXT_HAS_SELECTION!(state) {
            stb_textedit_delete_selection(str, state);
        } else {
            stb_textedit_clamp(str, state);
            if state.cursor > 0 {
                let prev = STB_TEXTEDIT_GETPREVCHARINDEX!(str, state.cursor);
                stb_textedit_delete(str, state, prev, state.cursor - prev);
                state.cursor = prev;
            }
        }
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_TEXTSTART {
        state.cursor = 0;
        state.select_start = 0;
        state.select_end = 0;
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_TEXTEND {
        state.cursor = STB_TEXTEDIT_STRINGLEN(str);
        state.select_start = 0;
        state.select_end = 0;
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_TEXTSTART | STB_TEXTEDIT_K_SHIFT {
        stb_textedit_prep_selection_at_cursor(state);
        state.cursor = 0;
        state.select_end = 0;
        state.has_preferred_x = 0;
    } else if key == (STB_TEXTEDIT_K_TEXTEND | STB_TEXTEDIT_K_SHIFT) {
        stb_textedit_prep_selection_at_cursor(state);
        state.cursor = STB_TEXTEDIT_STRINGLEN(str);
        state.select_end = STB_TEXTEDIT_STRINGLEN(str);
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_LINESTART {
        stb_textedit_clamp(str, state);
        stb_textedit_move_to_first(state);
        state.cursor = STB_TEXTEDIT_MOVELINESTART(str, state, state.cursor);
        state.has_preferred_x = 0;
    } else if key == STB_TEXTEDIT_K_LINEEND {
        stb_textedit_clamp(str, state);
        stb_textedit_move_to_last(str, state);
        state.cursor = STB_TEXTEDIT_MOVELINEEND(str, state, state.cursor);
        state.has_preferred_x = 0;
    } else if key == (STB_TEXTEDIT_K_LINESTART | STB_TEXTEDIT_K_SHIFT) {
        stb_textedit_clamp(str, state);
        stb_textedit_prep_selection_at_cursor(state);
        state.cursor = STB_TEXTEDIT_MOVELINESTART(str, state, state.cursor);
        state.select_end = state.cursor;
        state.has_preferred_x = 0;
    } else if key == (STB_TEXTEDIT_K_LINEEND | STB_TEXTEDIT_K_SHIFT) {
        stb_textedit_clamp(str, state);
        stb_textedit_prep_selection_at_cursor(state);
        state.cursor = STB_TEXTEDIT_MOVELINEEND(str, state, state.cursor);
        state.select_end = state.cursor;
        state.has_preferred_x = 0;
    } else {
        let c = STB_TEXTEDIT_KEYTOTEXT(key);
        if c > 0 {
            let ch = c as STB_TEXTEDIT_CHARTYPE;
            stb_textedit_text(str, state, &[ch]);
        }
    }
}

//#ifdef STB_TEXTEDIT_MOVEWORDLEFT
//      case STB_TEXTEDIT_K_WORDLEFT:
//         if (STB_TEXT_HAS_SELECTION(state))
//            stb_textedit_move_to_first(state);
//         else {
//            state->cursor = STB_TEXTEDIT_MOVEWORDLEFT(str, state->cursor);
//            stb_textedit_clamp( str, state );
//         }
//         break;

//      case STB_TEXTEDIT_K_WORDLEFT | STB_TEXTEDIT_K_SHIFT:
//         if( !STB_TEXT_HAS_SELECTION( state ) )
//            stb_textedit_prep_selection_at_cursor(state);

//         state->cursor = STB_TEXTEDIT_MOVEWORDLEFT(str, state->cursor);
//         state->select_end = state->cursor;

//         stb_textedit_clamp( str, state );
//         break;
//#endif

//#ifdef STB_TEXTEDIT_MOVEWORDRIGHT
//      case STB_TEXTEDIT_K_WORDRIGHT:
//         if (STB_TEXT_HAS_SELECTION(state))
//            stb_textedit_move_to_last(str, state);
//         else {
//            state->cursor = STB_TEXTEDIT_MOVEWORDRIGHT(str, state->cursor);
//            stb_textedit_clamp( str, state );
//         }
//         break;

//      case STB_TEXTEDIT_K_WORDRIGHT | STB_TEXTEDIT_K_SHIFT:
//         if( !STB_TEXT_HAS_SELECTION( state ) )
//            stb_textedit_prep_selection_at_cursor(state);

//         state->cursor = STB_TEXTEDIT_MOVEWORDRIGHT(str, state->cursor);
//         state->select_end = state->cursor;

//         stb_textedit_clamp( str, state );
//         break;
//#endif

/////////////////////////////////////////////////////////////////////////////
//
//      Undo processing
//
// @OPTIMIZE: the undo/redo buffer should be circular

pub fn stb_textedit_flush_redo(state: &mut StbUndoState) {
    state.redo_point = STB_TEXTEDIT_UNDOSTATECOUNT!();
    state.redo_char_point = STB_TEXTEDIT_UNDOCHARCOUNT!();
}

// discard the oldest entry in the undo list
pub fn stb_textedit_discard_undo(state: &mut StbUndoState) {
    if state.undo_point > 0 {
        // if the 0th undo state has characters, clean those up
        if state.undo_rec[0].char_storage >= 0 {
            let n = state.undo_rec[0].insert_length;
            let mut i;
            // delete n characters from all other records
            state.undo_char_point -= n;
            // STB_TEXTEDIT_memmove(state.undo_char, state.undo_char + n, (size_t) (state.undo_char_point*sizeof(IMSTB_TEXTEDIT_CHARTYPE)));
            STB_TEXTEDIT_memmove!(&mut state.undo_char, 0, n, state.undo_char_point);

            c_for!(i=0; i < state.undo_point; i += 1; {
               if state.undo_rec[i as usize].char_storage >= 0 {
                  state.undo_rec[i as usize].char_storage -= n; // @OPTIMIZE: get rid of char_storage and infer it
               }
            });
        }
        state.undo_point -= 1;

        // STB_TEXTEDIT_memmove(state.undo_rec, state.undo_rec+1, (size_t) (state.undo_point*sizeof(state.undo_rec[0])));
        // TODO
        STB_TEXTEDIT_memmove!(&mut state.undo_rec, 0, 1, state.undo_point);
    }
}

// discard the oldest entry in the redo list--it's bad if this
// ever happens, but because undo & redo have to store the actual
// characters in different cases, the redo character buffer can
// fill up even though the undo buffer didn't
pub fn stb_textedit_discard_redo(state: &mut StbUndoState) {
    let k = STB_TEXTEDIT_UNDOSTATECOUNT!() - 1;

    if state.redo_point <= k {
        // if the k'th undo state has characters, clean those up
        if state.undo_rec[k as usize].char_storage >= 0 {
            let n = state.undo_rec[k as usize].insert_length;
            let mut i;
            // move the remaining redo character data to the end of the buffer
            state.redo_char_point += n;
            // IMSTB_TEXTEDIT_memmove(state.undo_char + state.redo_char_point, state.undo_char + state.redo_char_point-n, (size_t) ((IMSTB_TEXTEDIT_UNDOCHARCOUNT - state.redo_char_point)*sizeof(IMSTB_TEXTEDIT_CHARTYPE)));
            STB_TEXTEDIT_memmove!(
                &mut state.undo_char,
                state.redo_char_point,
                state.redo_char_point - n,
                STB_TEXTEDIT_UNDOCHARCOUNT!() - state.redo_char_point
            );

            // adjust the position of all the other records to account for above memmove
            c_for!(i=state.redo_point; i < k; i += 1; {
               if state.undo_rec[i as usize].char_storage >= 0 {
                  state.undo_rec[i as usize].char_storage += n;
               }
            });
        }
        // now move all the redo records towards the end of the buffer; the first one is at 'redo_point'
        // [DEAR IMGUI]
        // let move_size = (size_t)((IMSTB_TEXTEDIT_UNDOSTATECOUNT - state.redo_point - 1) * sizeof(state.undo_rec[0]));
        // const char* buf_begin = (char*)state.undo_rec; (void)buf_begin;
        // const char* buf_end   = (char*)state.undo_rec + sizeof(state.undo_rec); (void)buf_end;
        // IM_ASSERT(((char*)(state.undo_rec + state.redo_point)) >= buf_begin);
        // IM_ASSERT(((char*)(state.undo_rec + state.redo_point + 1) + move_size) <= buf_end);
        // IMSTB_TEXTEDIT_memmove(state.undo_rec + state.redo_point+1, state.undo_rec + state.redo_point, move_size);
        // TODO:

        let move_count = STB_TEXTEDIT_UNDOSTATECOUNT!() - state.redo_point - 1;
        if move_count > 0 {
            STB_TEXTEDIT_memmove!(
                &mut state.undo_rec,
                state.redo_point + 1,
                state.redo_point,
                move_count
            );
        }

        // now move redo_point to point to the new one
        state.redo_point += 1;
    }
}

// TODO
pub fn stb_text_create_undo_record(state: &mut StbUndoState, numchars: int) -> Option<short> {
    // any time we create a new undo record, we discard redo
    stb_textedit_flush_redo(state);

    // if we have no free records, we have to make room, by sliding the
    // existing records down
    if state.undo_point == STB_TEXTEDIT_UNDOSTATECOUNT!() {
        stb_textedit_discard_undo(state);
    }

    // if the characters to store won't possibly fit in the buffer, we can't undo
    if numchars > STB_TEXTEDIT_UNDOCHARCOUNT!() {
        state.undo_point = 0;
        state.undo_char_point = 0;
        return None;
    }

    // if we don't have enough free characters in the buffer, we have to make room
    while state.undo_char_point + numchars > STB_TEXTEDIT_UNDOCHARCOUNT!() {
        stb_textedit_discard_undo(state);
    }

    let tmp = state.undo_point;
    state.undo_point += 1;
    // return &state.undo_rec[state.undo_point++];
    Some(tmp)
}

pub fn stb_text_createundo(
    state: &mut StbUndoState,
    pos: int,
    insert_len: int,
    delete_len: int,
) -> Option<&mut [STB_TEXTEDIT_CHARTYPE]> {
    // TODO
    // StbUndoRecord *r = stb_text_create_undo_record(state, insert_len);
    let r_indx = stb_text_create_undo_record(state, insert_len);
    let Some(r_indx) = r_indx else {
        return None;
    };

    let point = state.undo_char_point;

    let r = &mut state.undo_rec[r_indx as usize];
    r.location = pos;
    r.insert_length = insert_len;
    r.delete_length = delete_len;

    if insert_len == 0 {
        r.char_storage = -1;
        None
    } else {
        r.char_storage = point;
        state.undo_char_point += insert_len;
        // TODO
        Some(&mut state.undo_char[point as usize..])
    }
}

pub fn stb_text_undo(str: &mut STB_TEXTEDIT_STRING, state: &mut STB_TexteditState) {
    let s = &mut state.undostate;
    if s.undo_point == 0 {
        return;
    }

    // we need to do two things: apply the undo record, and create a redo record
    let u = s.undo_rec[s.undo_point as usize - 1];
    let mut r = &mut s.undo_rec[s.redo_point as usize - 1];
    r.char_storage = -1;

    r.insert_length = u.delete_length;
    r.delete_length = u.insert_length;
    r.location = u.location;

    if u.delete_length != 0 {
        // if the undo record says to delete characters, then the redo record will
        // need to re-insert the characters that get deleted, so we need to store
        // them.

        // there are three cases:
        //    there's enough room to store the characters
        //    characters stored for *redoing* don't leave room for redo
        //    characters stored for *undoing* don't leave room for redo
        // if the last is true, we have to bail

        if s.undo_char_point + u.delete_length >= STB_TEXTEDIT_UNDOCHARCOUNT!() {
            // the undo records take up too much character space; there's no space to store the redo characters
            r.insert_length = 0;
        } else {
            let mut i;

            // there's definitely room to store the characters eventually
            while s.undo_char_point + u.delete_length > s.redo_char_point {
                // should never happen:
                if s.redo_point == STB_TEXTEDIT_UNDOSTATECOUNT!() {
                    return;
                }
                // there's currently not enough room, so discard a redo record
                stb_textedit_discard_redo(s);
            }
            r = &mut s.undo_rec[s.redo_point as usize - 1];

            r.char_storage = s.redo_char_point - u.delete_length;
            s.redo_char_point -= u.delete_length;

            // now save the characters
            c_for!(i=0; i < u.delete_length; i += 1; {
               s.undo_char[(r.char_storage + i) as usize] = STB_TEXTEDIT_GETCHAR(str, u.location + i);
            });
        }

        // now we can carry out the deletion
        STB_TEXTEDIT_DELETECHARS(str, u.location, u.delete_length);
    }

    // check type of recorded action:
    if u.insert_length != 0 {
        // easy case: was a deletion, so we need to insert n characters
        STB_TEXTEDIT_INSERTCHARS(
            str,
            u.location,
            &s.undo_char[u.char_storage as usize..(u.char_storage + u.insert_length) as usize],
        );
        s.undo_char_point -= u.insert_length;
    }

    state.cursor = u.location + u.insert_length;

    s.undo_point -= 1;
    s.redo_point -= 1;
}

pub fn stb_text_redo(str: &mut STB_TEXTEDIT_STRING, state: &mut STB_TexteditState) {
    let s = &mut state.undostate;
    if s.redo_point == STB_TEXTEDIT_UNDOSTATECOUNT!() {
        return;
    }

    // we need to do two things: apply the redo record, and create an undo record
    let r = s.undo_rec[s.redo_point as usize];
    let u = &mut s.undo_rec[s.undo_point as usize];

    // we KNOW there must be room for the undo record, because the redo record
    // was derived from an undo record

    u.delete_length = r.insert_length;
    u.insert_length = r.delete_length;
    u.location = r.location;
    u.char_storage = -1;

    if r.delete_length != 0 {
        // the redo record requires us to delete characters, so the undo record
        // needs to store the characters

        if s.undo_char_point + u.insert_length > s.redo_char_point {
            u.insert_length = 0;
            u.delete_length = 0;
        } else {
            let mut i;
            u.char_storage = s.undo_char_point;
            s.undo_char_point += u.insert_length;

            // now save the characters
            c_for!(i=0; i < u.insert_length; i += 1; {
               s.undo_char[(u.char_storage + i) as usize] = STB_TEXTEDIT_GETCHAR(str, u.location + i);
            });
        }

        STB_TEXTEDIT_DELETECHARS(str, r.location, r.delete_length);
    }

    if r.insert_length != 0 {
        // easy case: need to insert n characters
        // STB_TEXTEDIT_INSERTCHARS(str, r.location, &s.undo_char[r.char_storage], r.insert_length);
        STB_TEXTEDIT_INSERTCHARS(
            str,
            r.location,
            &s.undo_char[r.char_storage as usize..(r.char_storage + r.insert_length) as usize],
        );
        s.redo_char_point += r.insert_length;
    }

    state.cursor = r.location + r.insert_length;

    s.undo_point += 1;
    s.redo_point += 1;
}

pub fn stb_text_makeundo_insert(state: &mut STB_TexteditState, location: int, length: int) {
    stb_text_createundo(&mut state.undostate, location, 0, length);
}

pub fn stb_text_makeundo_delete(
    str: &mut STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    location: int,
    length: int,
) {
    let mut i;
    let p = stb_text_createundo(&mut state.undostate, location, length, 0);
    if let Some(p) = p {
        c_for!(i=0; i < length; i+=1; {
           p[i as usize] = STB_TEXTEDIT_GETCHAR(str, location+i);
        });
    }
}

pub fn stb_text_makeundo_replace(
    str: &mut STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    location: int,
    old_length: int,
    new_length: int,
) {
    let mut i;
    let p = stb_text_createundo(&mut state.undostate, location, old_length, new_length);
    if let Some(p) = p {
        c_for!(i=0; i < old_length; i+=1; {
           p[i as usize] = STB_TEXTEDIT_GETCHAR(str, location+i);
        });
    }
}

// reset the state to default
pub fn stb_textedit_clear_state(state: &mut STB_TexteditState, is_single_line: int) {
    state.undostate.undo_point = 0;
    state.undostate.undo_char_point = 0;
    state.undostate.redo_point = STB_TEXTEDIT_UNDOSTATECOUNT!();
    state.undostate.redo_char_point = STB_TEXTEDIT_UNDOCHARCOUNT!();
    state.select_end = 0;
    state.select_start = 0;
    state.cursor = 0;
    state.has_preferred_x = 0;
    state.preferred_x = 0.0;
    state.cursor_at_end_of_line = 0;
    state.initialized = 1;
    state.single_line = is_single_line as unsigned_char;
    state.insert_mode = 0;
    state.row_count_per_page = 0;
}

// API initialize
pub fn stb_textedit_initialize_state(state: &mut STB_TexteditState, is_single_line: int) {
    stb_textedit_clear_state(state, is_single_line);
}

pub fn stb_textedit_paste(
    str: &mut STB_TEXTEDIT_STRING,
    state: &mut STB_TexteditState,
    text: &[STB_TEXTEDIT_CHARTYPE],
) -> int {
    stb_textedit_paste_internal(str, state, text)
}



/*
------------------------------------------------------------------------------
This software is available under 2 licenses -- choose whichever you prefer.
------------------------------------------------------------------------------
ALTERNATIVE A - MIT License
Copyright (c) 2017 Sean Barrett
Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
------------------------------------------------------------------------------
ALTERNATIVE B - Public Domain (www.unlicense.org)
This is free and unencumbered software released into the public domain.
Anyone is free to copy, modify, publish, use, compile, sell, or distribute this
software, either in source code form or as a compiled binary, for any purpose,
commercial or non-commercial, and by any means.
In jurisdictions that recognize copyright laws, the author or authors of this
software dedicate any and all copyright interest in the software to the public
domain. We make this dedication for the benefit of the public at large and to
the detriment of our heirs and successors. We intend this dedication to be an
overt act of relinquishment in perpetuity of all present and future rights to
this software under copyright law.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
------------------------------------------------------------------------------
*/

