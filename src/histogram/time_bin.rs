use anyhow::Result;
use std::sync::mpsc::{Receiver, Sender};

use crate::types::NormalizedTimeTag;

pub fn time_bin(
    receiver: Receiver<Vec<NormalizedTimeTag>>,
    mut sender: Sender<TimeBin>,
) -> Result<()> {
    todo!("Implement this function")
}

pub struct TimeBin {}
