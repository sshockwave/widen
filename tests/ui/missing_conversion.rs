use widen::Subset;

struct Missing;

#[derive(Subset)]
enum Source {
    Number(u8),
    Missing(Missing),
}

fn convert(source: Source) -> u16 {
    source.widen()
}

fn main() {}
