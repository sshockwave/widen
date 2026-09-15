use widen::Widen;

#[derive(Widen)]
enum Source {
    Number(u8),
}

fn convert(source: Result<(), Source>) -> Result<(), u16> {
    source.map_err(Widen::widen)?;
    Ok(())
}

fn main() {}
