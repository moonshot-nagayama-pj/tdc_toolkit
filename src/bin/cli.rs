use _mhtk_rs::mhlib_wrapper::*;

fn run(dev: u8) -> Result<(), String> {
    initialize(dev, Mode::T2, RefSource::InternalClock)?;
    println!("hw info: {:?}", get_hardware_info(dev)?);
    let num_inputs = get_number_of_input_channels(dev)? as u8;
    set_sync_divider(dev, 1)?;
    set_sync_edge_trigger(dev, -70, Edge::Falling)?;
    set_sync_channel_offset(dev, 0)?;
    for i in 0..num_inputs {
        set_input_edge_trigger(dev, i, -70, Edge::Falling)?;
        set_input_channel_offset(dev, i, 0)?;
    }

    start_measurement(dev, 5000)?;
    let mut buf: Vec<u32> = Vec::with_capacity(1048576);
    loop {
        let flags = get_flags(dev)?;
        if flags != 0 {
            println!("flags: {}", flags);
        }
        if (flags & 2) != 0 {
            println!("fifo over run");
            stop_measurement(dev)?;
            return Err("failed: fifo over run".to_string());
        }
        let size = read_fifo(dev, &mut buf[..])?;
        if size > 0 {
            println!("size: {}", size);
        } else {
            let status = ctc_status(dev)?;
            if status > 0 {
                println!("done");
                break;
            }
        }
    }
    stop_measurement(dev)?;
    Ok(())
}

fn main() -> Result<(), String> {
    println!("mhlib v{}", get_library_version().unwrap());
    let dev = 0;
    open_device(dev)?;
    let ret = run(dev);
    match ret {
        Ok(()) => println!("success"),
        Err(e) => println!("failed: {:?}", e),
    };
    close_device(dev)?;
    Ok(())
}
