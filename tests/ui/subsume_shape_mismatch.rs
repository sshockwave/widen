use widen::Widen;

enum Source {
    Value { first: u8, second: u8 },
}

#[derive(Widen)]
enum Destination {
    #[subsume(Source::Value)]
    Value { first: u8 },
}

fn main() {}
