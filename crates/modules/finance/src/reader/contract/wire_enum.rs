//! The macro behind every contract enum: serde spelling plus `id()` / `from_id()`.

/// Defines a string enum with an `id()` that is the exact wire spelling.
macro_rules! wire_enum {
    ($(#[$doc:meta])* $name:ident { $($(#[$vdoc:meta])* $variant:ident = $text:literal),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
        #[schema(rename_all = "snake_case")]
        pub enum $name {
            $($(#[$vdoc])* #[serde(rename = $text)] $variant),+
        }

        impl TryFrom<String> for $name {
            type Error = String;

            fn try_from(id: String) -> Result<Self, String> {
                Self::from_id(&id).ok_or_else(|| format!("unknown {}: {id}", stringify!($name)))
            }
        }

        impl $name {
            /// The spelling used on the wire and in the database.
            #[must_use]
            pub const fn id(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }

            /// The variant spelled `id`, if any.
            #[must_use]
            pub fn from_id(id: &str) -> Option<Self> {
                match id { $($text => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

pub(super) use wire_enum;
