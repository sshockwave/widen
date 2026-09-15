use widen::Widen;

enum Source {
    Shared,
    Missing,
}

#[derive(Widen)]
enum Destination {
    #[subsume(Source::Shared)]
    Shared,
}

fn main() {}
