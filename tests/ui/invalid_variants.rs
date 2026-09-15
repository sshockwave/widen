use widen::Widen;

#[derive(Widen)]
enum Unit {
    Missing,
}

#[derive(Widen)]
enum Tuple {
    Pair(u32, u32),
}

#[derive(Widen)]
enum Named {
    Pair { first: u32, second: u32 },
}

fn main() {}
