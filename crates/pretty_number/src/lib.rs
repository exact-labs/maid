use std::fmt::{self, Formatter};

pub fn pretty_number(mut n: u16) -> Number {
    let ones = digit(n % 10);
    n /= 10;
    let tens = digit(n % 10);
    n /= 10;
    let hundreds = digit(n % 10);
    n /= 10;
    let thousands = digit(n % 10);
    n /= 10;
    if n != 0 {
        panic!("number too large");
    }
    Number { thousands, hundreds, tens, ones }
}

/// panics on numbers greater than 9
fn digit(n: u16) -> Digit {
    match n {
        0 => Zero,
        1 => One,
        2 => Two,
        3 => Three,
        4 => Four,
        5 => Five,
        6 => Six,
        7 => Seven,
        8 => Eight,
        9 => Nine,
        _ => panic!("digit too large"),
    }
}

/// (up to) 4 digits number
pub struct Number {
    thousands: Digit,
    hundreds: Digit,
    tens: Digit,
    ones: Digit,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Digit {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}
use Digit::*;

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Zero => write!(f, "zero"),
            One => write!(f, "one"),
            Two => write!(f, "two"),
            Three => write!(f, "three"),
            Four => write!(f, "four"),
            Five => write!(f, "five"),
            Six => write!(f, "six"),
            Seven => write!(f, "seven"),
            Eight => write!(f, "eight"),
            Nine => write!(f, "nine"),
        }
    }
}

struct Tens(Digit);

impl fmt::Display for Tens {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Zero => Ok(()),
            One => Ok(()),
            Two => write!(f, "twenty"),
            Three => write!(f, "thirty"),
            Four => write!(f, "forty"),
            Five => write!(f, "fifty"),
            Six => write!(f, "sixty"),
            Seven => write!(f, "seventy"),
            Eight => write!(f, "eighty"),
            Nine => write!(f, "ninety"),
        }
    }
}

struct Teens(Digit);

impl fmt::Display for Teens {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Zero => write!(f, "ten"),
            One => write!(f, "eleven"),
            Two => write!(f, "twelve"),
            Three => write!(f, "thirteen"),
            Four => write!(f, "fourteen"),
            Five => write!(f, "fifteen"),
            Six => write!(f, "sixteen"),
            Seven => write!(f, "seventeen"),
            Eight => write!(f, "eighteen"),
            Nine => write!(f, "nineteen"),
        }
    }
}

#[derive(PartialEq, Eq)]
enum Space {
    None,
    One,
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Space::None => write!(f, ""),
            Space::One => write!(f, " "),
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut spacing = Space::None;
        if self.thousands != Zero {
            write!(f, "{} thousand", self.thousands)?;
            spacing = Space::One;
        }
        if self.hundreds != Zero {
            write!(f, "{}{} hundred", spacing, self.hundreds)?;
            spacing = Space::One;
        }
        match self.tens {
            Zero if self.ones == Zero && spacing == Space::One => Ok(()), // skip trailing 00
            Zero => write!(f, "{}{}", spacing, self.ones),
            One => write!(f, "{}{}", spacing, Teens(self.ones)),
            _ if self.ones == Zero => write!(f, "{}{}", spacing, Tens(self.tens)),
            _ => write!(f, "{}{}-{}", spacing, Tens(self.tens), self.ones),
        }
    }
}
