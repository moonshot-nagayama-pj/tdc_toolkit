//! Common, normalized types used to communicate across channels.

pub struct NormalizedTimeTag {
    pub channel_id: u16,

    /// The time tag, in picoseconds, counting up from the start of
    /// the measurement.
    pub time_tag_ps: u64,
}
