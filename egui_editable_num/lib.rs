use std::{fmt::Display, str::FromStr};

use egui::TextBuffer;

/// EditableOnText is a wrapper for a type that can be edited as text.
/// ```
///  if egui::TextEdit::singleline(input)
///     .clip_text(false)
///     .desired_width(0.0)
///     .margin(ui.spacing().item_spacing)
///     .show(ui)
///     .response
///     .lost_focus()
/// {
///     input.fmt();
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditableOnText<T: ToString + FromStr> {
    obj: T,
    str: String,
}

impl<T: ToString + FromStr + Default> Default for EditableOnText<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ToString + FromStr + Display> Display for EditableOnText<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.obj)
    }
}

impl<T: ToString + FromStr> EditableOnText<T> {
    pub fn new(obj: T) -> Self {
        let str = obj.to_string();

        Self { str, obj }
    }

    pub fn mut_ref(&mut self) -> &mut T {
        &mut self.obj
    }

    pub fn as_ref(&self) -> &T {
        &self.obj
    }

    pub fn try_update(&mut self) -> bool {
        if let Ok(obj) = self.str.parse() {
            self.obj = obj;
            true
        } else {
            false
        }
    }

    pub fn fmt(&mut self) {
        self.str = self.obj.to_string();
    }
}

impl<T: ToString + FromStr + Copy> EditableOnText<T> {
    pub fn get(&self) -> T {
        self.obj
    }
}

impl<T: ToString + FromStr> TextBuffer for EditableOnText<T> {
    fn is_mutable(&self) -> bool {
        true
    }

    fn as_str(&self) -> &str {
        &self.str
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        let ret = self.str.insert_text(text, char_index);

        self.try_update();

        ret
    }

    fn delete_char_range(&mut self, char_range: std::ops::Range<usize>) {
        self.str.delete_char_range(char_range);

        self.try_update();
    }
}
