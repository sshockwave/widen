use widen::Subset;

#[derive(Subset)]
enum Source {
    Number(u8),
}

fn convert(source: Result<(), Source>) -> Result<(), u16> {
    source.map_err(Subset::widen)?;
    Ok(())
}

fn main() {}
