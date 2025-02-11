use std::sync::mpsc;

// A tuple representing a single T2 record.
//
// The first int is the channel ID. Channel 0 is the sync channel: raw
// records sent with the "special" bit set indicating that they contain
// a sync timetag are translated to channel 0 here. To add to the
// confusion, "normal" channels are represented in the raw records
// starting from 0; they are shifted by 1 here, so channel 0 in the raw
// record becomes channel 1 here. Given that the MultiHarp has no more
// than 64 channels, an 8-bit unsigned int should be sufficient to
// represent this value.
//
// The second int is the time tag, in picoseconds. A 64-bit unsigned
// int should be sufficient for experiments of up to a few months in
// length. Note that the raw value is not in picoseconds; it has been
// converted here.
//
// A tuple is used to avoid performance penalties that would be caused
// by creating a new object for each record.
fn split_raw_t2_record(raw_record: u32) -> (u32, u32, u64) {
    let special = (raw_record >> 31) & 0x01; // highest bit
    let channel = (raw_record >> 25) & 0x3F; // next six bits
    let time_tag = raw_record & 0x1FFFFFF; // the rest
    (special, channel, time_tag as u64)
}

struct T2RecordChannelProcessor {
    // in time-tag units, e.g. one unit = 5 picoseconds when resolution is 5
    t2wraparound_v2: u64,
    // in time-tag units, e.g. one unit = 5 picoseconds when resolution is 5
    overflow_correction: u64,
    // in picoseconds
    resolution: u64,
}

impl T2RecordChannelProcessor {
    pub fn new() -> T2RecordChannelProcessor {
        T2RecordChannelProcessor {
            t2wraparound_v2: 33554432,
            overflow_correction: 0,
            resolution: 5,
        }
    }

    pub fn process(
        &mut self,
        rx_channel: mpsc::Receiver<Vec<u32>>,
        mut tx_channel: mpsc::Sender<(u16, u64)>,
    ) {
        for raw_records in rx_channel {
            self.process_raw_records(raw_records, &mut tx_channel);
        }
    }

    fn process_raw_records(
        &mut self,
        raw_records: Vec<u32>,
        tx_channel: &mut mpsc::Sender<(u16, u64)>,
    ) {
        for raw_record in raw_records.iter() {
            let (special, channel, time_tag) = split_raw_t2_record(*raw_record);
            if !self.process_special_records(special, channel, time_tag, tx_channel) {
                self.process_normal_record(channel, time_tag, tx_channel);
            }
        }
    }

    fn process_special_records(
        &mut self,
        special: u32,
        channel: u32,
        time_tag: u64,
        tx_channel: &mut mpsc::Sender<(u16, u64)>,
    ) -> bool {
        if special != 1 {
            return false;
        }
        if channel == 0x3F {
            // Overflow
            if time_tag == 0 {
                // old style overflow, shouldn't happen
                self.overflow_correction += self.t2wraparound_v2;
            } else {
                self.overflow_correction += self.t2wraparound_v2 * time_tag;
            }
            return true;
        }
        if channel == 0 {
            // Sync channel
            let true_time = self.overflow_correction + time_tag;
            tx_channel
                .send((0u16, (true_time * self.resolution)))
                .expect("failed to send message");
            return true;
        }
        // TODO Currently, this code discards external marker special records.
        //
        // Specifically, a channel between 1 and 15 inclusive indicates an external
        // marker; see the MultiHarp manual.
        true
    }

    fn process_normal_record(
        &self,
        channel: u32,
        time_tag: u64,
        tx_channel: &mut mpsc::Sender<(u16, u64)>,
    ) {
        let true_time = self.overflow_correction + time_tag;
        tx_channel
            .send(((channel as u16 + 1), (true_time * self.resolution)))
            .expect("failed to send message");
    }
}
