pub fn dotted(levels: usize) -> String {
    let mut out = String::with_capacity(levels * 2);
    for _ in 0..levels {
        out.push('·');
        out.push(' ');
    }
    out
}

pub fn spaced(levels: usize) -> String {
    let mut out = String::with_capacity(levels * 2);
    for _ in 0..levels {
        out.push(' ');
        out.push(' ');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotted_works() {
        assert_eq!(dotted(0), "");
        assert_eq!(dotted(1), "· ");
        assert_eq!(dotted(2), "· · ");
        assert_eq!(dotted(3), "· · · ");
    }

    #[test]
    fn spaced_works() {
        assert_eq!(spaced(0), "");
        assert_eq!(spaced(1), "  ");
        assert_eq!(spaced(2), "    ");
        assert_eq!(spaced(3), "      ");
    }
}
