/// Declare an enum with shorthand for variants named after their payload type.
///
/// A bare `ParseError` becomes `ParseError(ParseError)`. Explicit tuple and named
/// variants are preserved, as are the enum's attributes, visibility and generics.
/// This macro generates only the enum; apply derives and conversions as usual.
///
/// ```
/// struct ParseError;
///
/// widen::type_enum! {
///     enum Error {
///         ParseError,
///         Io(std::io::Error),
///     }
/// }
///
/// let error = Error::ParseError(ParseError);
/// ```
#[macro_export]
macro_rules! type_enum {
    ($(#[$attr:meta])* $vis:vis enum $name:ident $($tail:tt)*) => {
        $crate::type_enum!(@header [$(#[$attr])* $vis enum $name] $($tail)*);
    };
    (@header [$($header:tt)*] { $($variants:tt)* }) => {
        $crate::type_enum!(@variants [$($header)*] [] $($variants)* ,);
    };
    (@header [$($header:tt)*] $next:tt $($rest:tt)*) => {
        $crate::type_enum!(@header [$($header)* $next] $($rest)*);
    };
    (@variants [$($header:tt)*] [$($variants:tt)*]) => {
        $($header)* { $($variants)* }
    };
    (@variants [$($header:tt)*] [$($variants:tt)*] , $($rest:tt)*) => {
        $crate::type_enum!(@variants [$($header)*] [$($variants)*] $($rest)*);
    };
    (@variants [$($header:tt)*] [$($variants:tt)*]
        $(#[$attr:meta])* $name:ident ($($fields:tt)*) , $($rest:tt)*) => {
        $crate::type_enum!(@variants [$($header)*]
            [$($variants)* $(#[$attr])* $name($($fields)*),] $($rest)*);
    };
    (@variants [$($header:tt)*] [$($variants:tt)*]
        $(#[$attr:meta])* $name:ident { $($fields:tt)* } , $($rest:tt)*) => {
        $crate::type_enum!(@variants [$($header)*]
            [$($variants)* $(#[$attr])* $name { $($fields)* },] $($rest)*);
    };
    (@variants [$($header:tt)*] [$($variants:tt)*]
        $(#[$attr:meta])* $name:ident , $($rest:tt)*) => {
        $crate::type_enum!(@variants [$($header)*]
            [$($variants)* $(#[$attr])* $name($name),] $($rest)*);
    };
}
