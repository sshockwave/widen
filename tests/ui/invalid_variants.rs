use widen::Subset;

#[derive(Subset)]
enum Unit {
    Missing,
}

#[derive(Subset)]
enum Tuple {
    Pair(u32, u32),
}

#[derive(Subset)]
enum Named {
    Pair { first: u32, second: u32 },
}

fn main() {}
