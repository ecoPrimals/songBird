// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use std::fmt;
use std::hash::{Hash, Hasher};

/// **ZERO-COPY**: String reference that can be either borrowed or owned
#[derive(Debug, Clone)]
pub enum ZeroCopyString<'a> {
    /// Borrowed string - zero allocation
    Borrowed(&'a str),
    /// Owned string - allocated when necessary
    Owned(String),
    /// Static string - zero runtime cost
    Static(&'static str),
}

impl<'a> ZeroCopyString<'a> {
    /// Create from static string - zero cost
    #[inline]
    pub const fn from_static(s: &'static str) -> Self {
        Self::Static(s)
    }

    /// Create from borrowed string - zero allocation
    #[inline]
    pub fn from_borrowed(s: &'a str) -> Self {
        Self::Borrowed(s)
    }

    /// Create from owned string when necessary
    #[inline]
    pub fn from_owned(s: String) -> Self {
        Self::Owned(s)
    }

    /// Get string slice - zero cost operation
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(s) | Self::Static(s) => s,
            Self::Owned(s) => s.as_str(),
        }
    }

    /// Convert to owned string only when necessary
    pub fn into_owned(self) -> String {
        match self {
            Self::Borrowed(s) | Self::Static(s) => s.to_string(),
            Self::Owned(s) => s,
        }
    }

    /// Check if string is empty - zero cost
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    /// Get string length - zero cost
    #[inline]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }
}

impl PartialEq for ZeroCopyString<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for ZeroCopyString<'_> {}

impl<'a> From<&'a str> for ZeroCopyString<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        Self::Borrowed(s)
    }
}

impl From<String> for ZeroCopyString<'_> {
    #[inline]
    fn from(s: String) -> Self {
        Self::Owned(s)
    }
}

impl AsRef<str> for ZeroCopyString<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Hash for ZeroCopyString<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl fmt::Display for ZeroCopyString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
