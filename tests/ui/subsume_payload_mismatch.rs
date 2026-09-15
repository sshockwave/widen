use widen::Widen;

struct Payload;
struct Converted;

impl From<Payload> for Converted {
    fn from(_: Payload) -> Self {
        Self
    }
}

enum Source {
    Value(Payload),
}

#[derive(Widen)]
enum Destination {
    // A From implementation must not enable conversion without from(...).
    #[subsume(Source::Value)]
    Value(Converted),
}

fn main() {}
