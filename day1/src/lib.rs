pub fn digit<T: AsRef<str>>(s: T) -> Option<u32> {
    let s = s.as_ref();
    let mut it = s.bytes();

    let first = (it.find(|b| b.is_ascii_digit())? - 0x30) as u32;
    let second = it
        .filter(|b| b.is_ascii_digit())
        .last()
        .map(|x| (x - 0x30) as u32)
        .unwrap_or(first);

    Some((first * 10) + second)
}

pub fn text_to_digit<T: AsRef<str>>(s: T) -> String {
    let s = s.as_ref().to_string();

    (0..s.len())
        .map(|head| match &s[head..] {
            m if m.starts_with("one") => "1",
            m if m.starts_with("two") => "2",
            m if m.starts_with("three") => "3",
            m if m.starts_with("four") => "4",
            m if m.starts_with("five") => "5",
            m if m.starts_with("six") => "6",
            m if m.starts_with("seven") => "7",
            m if m.starts_with("eight") => "8",
            m if m.starts_with("nine") => "9",
            m => &m[0..1],
        })
        .collect()
}

/// Expected solution doesn't prefer first, it keeps both values in an overlap
pub fn text_to_digit_no_overlap<T: AsRef<str>>(s: T) -> String {
    let mut s = s.as_ref().to_string();
    let mut head = 0;

    while head <= (s.len().saturating_sub(3)) {
        let m = match &s[head..] {
            m if m.starts_with("one") => Some((1, 3)),
            m if m.starts_with("two") => Some((2, 3)),
            m if m.starts_with("three") => Some((3, 5)),
            m if m.starts_with("four") => Some((4, 4)),
            m if m.starts_with("five") => Some((5, 4)),
            m if m.starts_with("six") => Some((6, 3)),
            m if m.starts_with("seven") => Some((7, 5)),
            m if m.starts_with("eight") => Some((8, 5)),
            m if m.starts_with("nine") => Some((9, 4)),
            _ => None,
        };

        if let Some((digit, len)) = m {
            s = s[0..head].to_string() + &digit.to_string() + &s[head + len..];
        }

        head += 1;
    }

    s
}

/// Expected solution parses in order of occurrence, not value
pub fn text_to_digit_value_ordered<T: AsRef<str>>(s: T) -> String {
    s.as_ref()
        .replace("one", "1")
        .replace("two", "2")
        .replace("three", "3")
        .replace("four", "4")
        .replace("five", "5")
        .replace("six", "6")
        .replace("seven", "7")
        .replace("eight", "8")
        .replace("nine", "9")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_first() {
        assert_eq!(digit("1abc2").unwrap(), 12);
    }

    #[test]
    fn parse_last() {
        assert_eq!(digit("treb7uchet").unwrap(), 77);
    }

    #[test]
    fn parse_and_sum() {
        assert_eq!(
            ["1abc2", "pqr3stu8vwx", "a1b2c3d4e5f", "treb7uchet"]
                .into_iter()
                .map(digit)
                .collect::<Option<Vec<_>>>()
                .unwrap()
                .into_iter()
                .sum::<u32>(),
            142
        )
    }

    #[test]
    fn conv_to_digit() {
        assert_eq!(text_to_digit("two1nine"), "2wo19ine");
        assert_eq!(text_to_digit("eightwothree"), "8igh2wo3hree");
    }

    #[test]
    fn sum_conv_to_digit() {
        assert_eq!(
            [
                "two1nine",
                "eightwothree",
                "abcone2threexyz",
                "xtwone3four",
                "4nineeightseven2",
                "zoneight234",
                "7pqrstsixteen",
            ]
            .into_iter()
            .map(text_to_digit)
            .map(digit)
            .collect::<Option<Vec<_>>>()
            .unwrap()
            .into_iter()
            .sum::<u32>(),
            281
        )
    }
}
