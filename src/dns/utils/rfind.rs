pub trait RFindNth {
    fn rfind_nth(&self, needle: char, n: usize) -> Option<usize>;
}

impl RFindNth for str {
    fn rfind_nth(&self, needle: char, mut n: usize) -> Option<usize> {
        for (i, c) in self.char_indices().rev() {
            if c == needle {
                if n == 0 {
                    return Some(i);
                } else {
                    n -= 1;
                }
            }
        }
        None
    }
}
