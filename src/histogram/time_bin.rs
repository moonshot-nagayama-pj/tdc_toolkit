use anyhow::Result;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use crate::types::NormalizedTimeTag;

const BIN_WIDTH_PS: u64 = 1_000_000; // example: 1 microsecond = 1,000,000 ps

pub fn time_bin(receiver: Receiver<Vec<NormalizedTimeTag>>, sender: Sender<TimeBin>) -> Result<()> {
    let mut current_bin_start: Option<u64> = None;
    let mut current_counts: HashMap<u16, u64> = HashMap::new();

    for batch in receiver {
        for event in batch {
            let event_bin_start = (event.time_tag_ps / BIN_WIDTH_PS) * BIN_WIDTH_PS;

            match current_bin_start {
                None => {
                    current_bin_start = Some(event_bin_start);
                }
                Some(start) if event_bin_start == start => {
                    // same bin, continue
                }
                Some(start) if event_bin_start > start => {
                    // flush current bin
                    sender.send(TimeBin {
                        start_time_ps: start,
                        end_time_ps: start + BIN_WIDTH_PS,
                        counts: std::mem::take(&mut current_counts),
                    })?;

                    // if there are skipped bins, emit empty ones too
                    let mut next_start = start + BIN_WIDTH_PS;
                    while next_start < event_bin_start {
                        sender.send(TimeBin {
                            start_time_ps: next_start,
                            end_time_ps: next_start + BIN_WIDTH_PS,
                            counts: HashMap::new(),
                        })?;
                        next_start += BIN_WIDTH_PS;
                    }

                    current_bin_start = Some(event_bin_start);
                }
                Some(_) => {
                    // older event arrived after newer ones
                    // for now, just ignore this case because input is expected to be time-ordered
                }
            }

            *current_counts.entry(event.channel_id).or_insert(0) += 1;
        }
    }

    // flush the last bin after the channel closes
    if let Some(start) = current_bin_start {
        sender.send(TimeBin {
            start_time_ps: start,
            end_time_ps: start + BIN_WIDTH_PS,
            counts: current_counts,
        })?;
    }

    Ok(())
}

pub struct TimeBin {
    pub start_time_ps: u64,
    pub end_time_ps: u64,
    pub counts: HashMap<u16, u64>,
}
