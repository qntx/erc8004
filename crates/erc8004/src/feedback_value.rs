//! `(value: int128, decimals: u8)` — the contract's feedback magnitude.

/// Decimal-string or `(value, decimals)` pair validation failure.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FeedbackValueError {
    /// String does not match `-?[0-9]+(\.[0-9]+)?`.
    #[error("invalid decimal string")]
    InvalidDecimalString,
    /// More than 18 fraction digits.
    #[error("too many decimals")]
    TooManyDecimals,
    /// `|value| > 1e38`.
    #[error("value out of range")]
    ValueOutOfRange,
}

impl FeedbackValueError {
    /// The shared error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidDecimalString => "InvalidDecimalString",
            Self::TooManyDecimals => "TooManyDecimals",
            Self::ValueOutOfRange => "ValueOutOfRange",
        }
    }
}

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

const MAX_DECIMALS: u8 = 18;
const MAX_VALUE: i128 = 10i128.pow(38);

/// A validated `(value, decimals)` pair: `decimals <= 18`, `|value| <= 1e38`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeedbackValue {
    value: i128,
    decimals: u8,
}

impl FeedbackValue {
    /// Checked constructor: `decimals <= 18` and `|value| <= 1e38`.
    ///
    /// # Errors
    ///
    /// `TooManyDecimals` when `decimals > 18`, `ValueOutOfRange` when `|value| > 1e38`.
    pub const fn new(value: i128, decimals: u8) -> Result<Self, FeedbackValueError> {
        if decimals > MAX_DECIMALS {
            return Err(FeedbackValueError::TooManyDecimals);
        }
        if value.unsigned_abs() > MAX_VALUE as u128 {
            return Err(FeedbackValueError::ValueOutOfRange);
        }
        Ok(Self { value, decimals })
    }

    /// The `int128` magnitude in units of `10^-decimals`.
    #[must_use]
    pub const fn value(&self) -> i128 {
        self.value
    }

    /// The `uint8` decimal places (0..=18).
    #[must_use]
    pub const fn decimals(&self) -> u8 {
        self.decimals
    }
}

/// Parses `-?[0-9]+(\.[0-9]+)?` — leading zeros allowed; fraction digits kept as written.
impl FromStr for FeedbackValue {
    type Err = FeedbackValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (negative, body) = text
            .strip_prefix('-')
            .map_or((false, text), |rest| (true, rest));
        let syntax_ok = !body.is_empty()
            && body.split('.').count() <= 2
            && body
                .split('.')
                .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
        if !syntax_ok {
            return Err(FeedbackValueError::InvalidDecimalString);
        }
        let mut split = body.splitn(2, '.');
        let int_part = split.next().unwrap_or("");
        let frac_part = split.next().unwrap_or("");
        if frac_part.len() > MAX_DECIMALS as usize {
            return Err(FeedbackValueError::TooManyDecimals);
        }
        let digits: String = format!("{int_part}{frac_part}");
        let mut value: i128 = digits
            .parse()
            .map_err(|_| FeedbackValueError::ValueOutOfRange)?;
        if negative && value != 0 {
            value = -value;
        }
        Self::new(
            value,
            u8::try_from(frac_part.len()).map_err(|_| FeedbackValueError::TooManyDecimals)?,
        )
    }
}

/// Prints exactly `decimals` fraction digits: `(0,2)` → `"0.00"`, `(-5,2)` → `"-0.05"`.
impl fmt::Display for FeedbackValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.decimals == 0 {
            return write!(f, "{}", self.value);
        }
        let negative = self.value < 0;
        let digits = format!(
            "{:0>width$}",
            self.value.unsigned_abs(),
            width = self.decimals as usize + 1
        );
        let split = digits.len() - self.decimals as usize;
        let (whole, fraction) = digits.split_at(split);
        write!(
            f,
            "{}{}.{}",
            if negative { "-" } else { "" },
            whole,
            fraction
        )
    }
}
