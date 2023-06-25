use std::fmt::Display;
use std::num::TryFromIntError;
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
    #[error("failed to clean up terminal: {0}")]
    CleanUpTerminal(#[source] io::Error),

    #[error("failed to serialize JSON: {0}")]
    SerializeJson(#[source] serde_json::Error),

    #[error("failed to wrote file: {0}")]
    WriteFile(#[source] io::Error),

/// The Unix file mode. The special mode `0` indicates that the file did not exist.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FileMode(pub usize);

impl FileMode {
    /// Get the file mode corresponding to an absent file. This typically
    /// indicates that the file was created (if the "before" mode) or deleted
    /// (if the "after" mode).
    pub fn absent() -> Self {
        Self(0)
    }
}

impl Display for FileMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(mode) = self;
        write!(f, "{mode:o}")
    }
}

impl From<usize> for FileMode {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<FileMode> for usize {
    fn from(value: FileMode) -> Self {
        let FileMode(value) = value;
        value
    }
}

impl TryFrom<u32> for FileMode {
    type Error = TryFromIntError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl TryFrom<FileMode> for u32 {
    type Error = TryFromIntError;

    fn try_from(value: FileMode) -> Result<Self, Self::Error> {
        let FileMode(value) = value;
        value.try_into()
    }
}

impl TryFrom<i32> for FileMode {
    type Error = TryFromIntError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl TryFrom<FileMode> for i32 {
    type Error = TryFromIntError;

    fn try_from(value: FileMode) -> Result<Self, Self::Error> {
        let FileMode(value) = value;
        value.try_into()
    }
}
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
    /// The Unix file mode of the file (before any changes), if available. This
    /// may be rendered by the UI.
    /// user-provided updated to the file mode.
/// The contents of a file selected as part of the record operation.
#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum SelectedContents<'a> {
    /// The file didn't exist or was deleted.
    Absent,
    /// The file contents have not changed.
    Unchanged,
    /// The file contains binary contents.
    Binary {
        /// The UI description of the old version of the file.
        old_description: Option<Cow<'a, str>>,
        /// The UI description of the new version of the file.
        new_description: Option<Cow<'a, str>>,
    },

    /// The file contained the following text contents.
    Present {
        /// The contents of the file.
        contents: String,
    },
}

impl SelectedContents<'_> {
    fn push_str(&mut self, s: &str) {
        match self {
            SelectedContents::Absent | SelectedContents::Unchanged => {
                *self = SelectedContents::Present {
                    contents: s.to_owned(),
                };
            }
            SelectedContents::Binary {
                old_description: _,
                new_description: _,
            } => {
                // Do nothing.
            }
            SelectedContents::Present { contents } => {
                contents.push_str(s);
            }
        }
}
impl File<'_> {
                }
                | Section::Binary { .. } => None,
    pub fn get_selected_contents(&self) -> (SelectedContents, SelectedContents) {
        let mut acc_selected = SelectedContents::Unchanged;
        let mut acc_unselected = SelectedContents::Unchanged;


                    is_toggled,
                    before,
                    after,
                } => {
                    if *is_toggled && after == &FileMode::absent() {
                        acc_selected = SelectedContents::Absent;
                    } else if !is_toggled && before == &FileMode::absent() {
                        acc_unselected = SelectedContents::Absent;
                    }
                }

                Section::Binary {
                    is_toggled,
                    old_description,
                    new_description,
                    let selected_contents = SelectedContents::Binary {
                        old_description: old_description.clone(),
                        new_description: new_description.clone(),
                    };
                    if *is_toggled {
                        acc_selected = selected_contents;
                    } else {
                        acc_unselected = selected_contents;
                    }
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        /// The contents of the lines, including their trailing newline
        /// character(s), if any.
        /// The contents of the lines, including their trailing newline
        /// character(s), if any.

    /// This file contains binary contents.
    Binary {
        /// Whether or not the binary contents change was accepted.
        is_toggled: bool,

        /// The description of the old binary contents, for use in the UI only.
        old_description: Option<Cow<'a, str>>,

        /// The description of the new binary contents, for use in the UI only.
        new_description: Option<Cow<'a, str>>,
    },
            Section::Changed { .. } | Section::FileMode { .. } | Section::Binary { .. } => true,
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]