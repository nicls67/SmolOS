use core::fmt::Display;

/// A wrapper struct representing a duration in milliseconds.
///
/// The `Milliseconds` struct is a simple wrapper around a `u32` value, allowing
/// you to explicitly work with values representing time durations in milliseconds.
///
/// # Fields
///
/// * `0` - The inner `u32` value representing the duration in milliseconds.
///
pub struct Milliseconds(pub u32);

impl Display for Milliseconds {
    /// Implements the `fmt` method for formatting the display of the implementing type.
    ///
    /// # Parameters
    /// - `&self`: A reference to the instance of the type implementing this method.
    /// - `f`: A mutable reference to the `core::fmt::Formatter` used to format the output.
    ///
    /// # Returns
    /// A `core::fmt::Result`, which indicates whether the formatting operation was successful.
    ///
    /// # Behavior
    /// This method formats the instance by writing a string representation of `self.0`
    /// followed by the " ms" suffix into the provided formatter. The `self.0` refers to
    /// the inner value assumed to be a numeric type representing milliseconds.
    ///
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ms", self.0)
    }
}

impl Milliseconds {
    /// Converts a `Seconds` value into `Milliseconds`.
    ///
    /// This function takes a `Seconds` type and converts it into its equivalent
    /// value in `Milliseconds` by multiplying the seconds value by 1000.
    ///
    /// # Parameters
    /// - `seconds`: A `Seconds` type containing the value to be converted into milliseconds.
    ///
    /// # Returns
    /// - A `Milliseconds` type representing the equivalent value of the input in milliseconds.
    ///
    pub fn from_seconds(seconds: Seconds) -> Self {
        Milliseconds(seconds.0 * 1000)
    }
    /// Converts the value of the current instance into a `u32`.
    ///
    /// # Returns
    /// - A `u32` representation of the wrapped value.
    ///
    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

/// A wrapper struct representing time in seconds.
///
/// `Seconds` is a newtype around a 32-bit unsigned integer (`u32`) that encapsulates
/// time measured in seconds. This struct can be useful for improving type safety and
/// code readability when working with time-related operations in your application.
///
/// # Fields
///
/// * `0`: The inner `u32` value representing the time in seconds.
///
/// Note: The maximum value is `u32::MAX` seconds (approximately 136 years).
///
pub struct Seconds(pub u32);

impl Display for Seconds {
    /// Formats the current instance for display purposes.
    ///
    /// This implementation of the `fmt` method is for the `Display` trait.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to the `Formatter` that provides an output stream
    ///         for writing the formatted string.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating whether the formatting operation was successful or not:
    /// * `Ok(())` means the operation succeeded.
    /// * `Err` contains a formatting error if one occurred.
    ///
    /// # Behavior
    ///
    /// This method writes the string representation of the first element of the struct
    /// (`self.0`) followed by a space and the letter "s" into the provided formatter.
    ///
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} s", self.0)
    }
}

impl Seconds {
    /// Converts the value stored in the current instance to milliseconds.
    ///
    /// # Returns
    ///
    /// A `Milliseconds` object that represents the current value multiplied by 1000,
    /// effectively converting it to milliseconds.
    ///
    pub fn to_millis(&self) -> Milliseconds {
        Milliseconds(self.0 * 1000)
    }
    /// Converts the value of the current instance to a `u32`.
    ///
    /// # Returns
    ///
    /// A `u32` representation of the inner value.
    ///
    /// This method assumes that the inner value of the type can be directly represented as a `u32`.
    pub fn to_u32(&self) -> u32 {
        self.0
    }
}
