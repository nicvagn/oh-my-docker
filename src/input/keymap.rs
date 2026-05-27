use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Clone, Debug)]
pub struct ParsedKey {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl ParsedKey {
    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if self.code == code {
            return self.modifiers == modifiers
                || (self.modifiers == KeyModifiers::NONE && modifiers == KeyModifiers::SHIFT)
                || (self.modifiers == KeyModifiers::SHIFT && modifiers == KeyModifiers::NONE);
        }
        false
    }
}

/// Parse a keybinding string like "j", "Ctrl+A", "Esc", "Enter" into a ParsedKey.
pub fn parse_keybinding(s: &str) -> ParsedKey {
    // Space must be checked before trim (trim removes spaces)
    if s == " " || s.eq_ignore_ascii_case("space") {
        return ParsedKey {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
        };
    }
    let s = s.trim();
    match s.to_lowercase().as_str() {
        "esc" | "escape" => ParsedKey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
        },
        "enter" => ParsedKey {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
        },
        "tab" => ParsedKey {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
        },
        "backspace" => ParsedKey {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
        },
        "up" | "arrowup" | "arrow_up" => ParsedKey {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
        },
        "down" | "arrowdown" | "arrow_down" => ParsedKey {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
        },
        "left" | "arrowleft" | "arrow_left" => ParsedKey {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
        },
        "right" | "arrowright" | "arrow_right" => ParsedKey {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
        },
        "pageup" | "page_up" => ParsedKey {
            code: KeyCode::PageUp,
            modifiers: KeyModifiers::NONE,
        },
        "pagedown" | "page_down" => ParsedKey {
            code: KeyCode::PageDown,
            modifiers: KeyModifiers::NONE,
        },
        "ctrl+a" | "ctrl_a" => ParsedKey {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+b" | "ctrl_b" => ParsedKey {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+c" | "ctrl_c" => ParsedKey {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+d" | "ctrl_d" => ParsedKey {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+e" | "ctrl_e" => ParsedKey {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+f" | "ctrl_f" => ParsedKey {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+h" | "ctrl_h" => ParsedKey {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+s" | "ctrl_s" => ParsedKey {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+u" | "ctrl_u" => ParsedKey {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+y" | "ctrl_y" => ParsedKey {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+o" | "ctrl_o" => ParsedKey {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+p" | "ctrl_p" => ParsedKey {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+t" | "ctrl_t" => ParsedKey {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::CONTROL,
        },
        _ => {
            // Try to parse as a single character
            let chars: Vec<char> = s.chars().collect();
            if chars.len() == 1 {
                ParsedKey {
                    code: KeyCode::Char(chars[0]),
                    modifiers: if chars[0].is_uppercase() { KeyModifiers::SHIFT } else { KeyModifiers::NONE },
                }
            } else {
                // Default to the first character if it's not a special key
                ParsedKey {
                    code: KeyCode::Char(s.chars().next().unwrap_or('j')),
                    modifiers: KeyModifiers::NONE,
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyMap {
    pub quit: Vec<ParsedKey>,
    pub back: Vec<ParsedKey>,
    pub help: Vec<ParsedKey>,
    pub navigate_down: Vec<ParsedKey>,
    pub navigate_up: Vec<ParsedKey>,
    pub open_details: Vec<ParsedKey>,
    pub open_logs: Vec<ParsedKey>,
    pub open_shell: Vec<ParsedKey>,
    pub start_stop: Vec<ParsedKey>,
    pub restart: Vec<ParsedKey>,
    pub delete: Vec<ParsedKey>,
    pub search: Vec<ParsedKey>,
    pub toggle_selection: Vec<ParsedKey>,
    pub select_all: Vec<ParsedKey>,
    pub navigate_images: Vec<ParsedKey>,
    pub run_image: Vec<ParsedKey>,
    pub remove_image: Vec<ParsedKey>,
    pub remove_dangling_images: Vec<ParsedKey>,
    pub prune_images: Vec<ParsedKey>,
    pub events_export: Vec<ParsedKey>,
    pub jump_top: Vec<ParsedKey>,
    pub jump_bottom: Vec<ParsedKey>,
    pub statistics_sort: Vec<ParsedKey>,
    pub statistics_sort_desc: Vec<ParsedKey>,
    pub logs_export: Vec<ParsedKey>,
    pub toggle_timestamps: Vec<ParsedKey>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            quit: vec![parse_keybinding("q")],
            back: vec![parse_keybinding("Esc")],
            help: vec![parse_keybinding("?")],
            navigate_down: vec![parse_keybinding("j"), parse_keybinding("Down")],
            navigate_up: vec![parse_keybinding("k"), parse_keybinding("Up")],
            open_details: vec![parse_keybinding("Enter")],
            open_logs: vec![parse_keybinding("l")],
            open_shell: vec![parse_keybinding("s")],
            start_stop: vec![parse_keybinding("t")],
            restart: vec![parse_keybinding("r")],
            delete: vec![parse_keybinding("d")],
            search: vec![parse_keybinding("/")],
            toggle_selection: vec![parse_keybinding(" ")],
            select_all: vec![parse_keybinding("Ctrl+A")],
            navigate_images: vec![parse_keybinding("j"), parse_keybinding("Down")],
            run_image: vec![parse_keybinding("r")],
            remove_image: vec![parse_keybinding("d")],
            remove_dangling_images: vec![parse_keybinding("D")],
            prune_images: vec![parse_keybinding("p")],
            events_export: vec![parse_keybinding("s")],
            jump_top: vec![parse_keybinding("g")],
            jump_bottom: vec![parse_keybinding("G")],
            statistics_sort: vec![parse_keybinding("s")],
            statistics_sort_desc: vec![parse_keybinding("t")],
            logs_export: vec![parse_keybinding("Ctrl+S")],
            toggle_timestamps: vec![parse_keybinding("T")],
        }
    }
}

fn any_match(keys: &[ParsedKey], code: KeyCode, modifiers: KeyModifiers) -> bool {
    keys.iter().any(|k| k.matches(code, modifiers))
}

#[allow(dead_code)]
impl KeyMap {
    pub fn from_keybindings(keybindings: &crate::config::Keybindings) -> Self {
        fn parse_all(keys: &[String]) -> Vec<ParsedKey> {
            keys.iter().map(|s| parse_keybinding(s)).collect()
        }
        Self {
            quit: parse_all(&keybindings.quit),
            back: parse_all(&keybindings.back),
            help: parse_all(&keybindings.help),
            navigate_down: parse_all(&keybindings.navigate_down),
            navigate_up: parse_all(&keybindings.navigate_up),
            open_details: parse_all(&keybindings.open_details),
            open_logs: parse_all(&keybindings.open_logs),
            open_shell: parse_all(&keybindings.open_shell),
            start_stop: parse_all(&keybindings.start_stop),
            restart: parse_all(&keybindings.restart),
            delete: parse_all(&keybindings.delete),
            search: parse_all(&keybindings.search),
            toggle_selection: parse_all(&keybindings.toggle_selection),
            select_all: parse_all(&keybindings.select_all),
            navigate_images: parse_all(&keybindings.navigate_images),
            run_image: parse_all(&keybindings.run_image),
            remove_image: parse_all(&keybindings.remove_image),
            remove_dangling_images: parse_all(&keybindings.remove_dangling_images),
            prune_images: parse_all(&keybindings.prune_images),
            events_export: parse_all(&keybindings.events_export),
            jump_top: parse_all(&keybindings.jump_top),
            jump_bottom: parse_all(&keybindings.jump_bottom),
            statistics_sort: parse_all(&keybindings.statistics_sort),
            statistics_sort_desc: parse_all(&keybindings.statistics_sort_desc),
            logs_export: parse_all(&keybindings.logs_export),
            toggle_timestamps: parse_all(&keybindings.toggle_timestamps),
        }
    }

    pub fn matches(&self, key: &crossterm::event::KeyEvent) -> bool {
        self.matches_code(key.code, key.modifiers)
    }

    pub fn matches_code(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if any_match(&self.quit, code, modifiers) { return true; }
        if any_match(&self.back, code, modifiers) { return true; }
        if any_match(&self.help, code, modifiers) { return true; }
        false
    }

    pub fn is_navigate_down(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.navigate_down, code, modifiers)
    }

    pub fn is_navigate_up(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.navigate_up, code, modifiers)
    }

    pub fn is_open_details(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.open_details, code, modifiers)
    }

    pub fn is_open_logs(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.open_logs, code, modifiers)
    }

    pub fn is_open_shell(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.open_shell, code, modifiers)
    }

    pub fn is_start_stop(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.start_stop, code, modifiers)
    }

    pub fn is_restart(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.restart, code, modifiers)
    }

    pub fn is_delete(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.delete, code, modifiers)
    }

    pub fn is_search(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.search, code, modifiers)
    }

    pub fn is_toggle_selection(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.toggle_selection, code, modifiers)
    }

    pub fn is_select_all(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.select_all, code, modifiers)
    }

    pub fn is_navigate_images(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.navigate_images, code, modifiers)
    }

    pub fn is_run_image(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.run_image, code, modifiers)
    }

    pub fn is_remove_image(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.remove_image, code, modifiers)
    }

    pub fn is_remove_dangling_images(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.remove_dangling_images, code, modifiers)
    }

    pub fn is_prune_images(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.prune_images, code, modifiers)
    }

    pub fn is_jump_top(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.jump_top, code, modifiers)
    }

    pub fn is_jump_bottom(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.jump_bottom, code, modifiers)
    }

    pub fn is_statistics_sort(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.statistics_sort, code, modifiers)
    }

    pub fn is_statistics_sort_desc(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.statistics_sort_desc, code, modifiers)
    }

    pub fn is_logs_export(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.logs_export, code, modifiers)
    }

    pub fn is_events_export(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.events_export, code, modifiers)
    }

    pub fn is_toggle_timestamps(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.toggle_timestamps, code, modifiers)
    }

    pub fn is_help(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.help, code, modifiers)
    }

    pub fn is_quit(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.quit, code, modifiers)
    }

    pub fn is_back(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        any_match(&self.back, code, modifiers)
    }
}
