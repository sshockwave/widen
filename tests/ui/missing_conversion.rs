use widen::Widen;

struct Missing;

#[derive(Widen)]
enum Source {
    Number(u8),
    Missing(Missing),
}

fn convert(source: Source) -> u16 {
    source.widen()
}

fn main() {}
