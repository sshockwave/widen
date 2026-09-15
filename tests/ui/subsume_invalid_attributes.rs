use widen::Widen;

enum Source {
    Unit,
    Value(u8),
}

#[derive(Widen)]
enum Empty {
    #[subsume()]
    Unit,
}

#[derive(Widen)]
enum Bare {
    #[subsume(Unit)]
    Unit,
}

#[derive(Widen)]
enum Modifier {
    #[subsume(into(Source::Value))]
    Value(u8),
}

#[derive(Widen)]
enum UnitConversion {
    #[subsume(from(Source::Unit))]
    Unit,
}

#[derive(Widen)]
enum Duplicate {
    #[subsume(Source::Unit)]
    First,
    #[subsume(Source::Unit)]
    Second,
}

#[derive(Widen)]
#[subsume(Source::Unit)]
enum OnEnum {
    Unit,
}

#[derive(Widen)]
enum OnField {
    Value(#[subsume(Source::Value)] u8),
}

fn main() {}
