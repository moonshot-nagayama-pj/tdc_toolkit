from typing import TypedDict
import multiharp_toolkit._mhtk_rs as mh
import sys
import time
import pyarrow as pa

T2WRAPAROUND_V1 = 33552000
T2WRAPAROUND_V2 = 33554432
TimeTagDataSchema = pa.schema([("ch", pa.int8()), ("timestamp", pa.float64())])


class DeviceInputChannelConfig(TypedDict):
    edge_trigger_level: int
    edge_trigger: "mh.Edge"
    channel_offset: int
    enable: bool


class DeviceConfig(TypedDict):
    sync_divider: int
    sync_edge_trigger_level: int  # mV
    sync_edge: "mh.Edge"
    sync_channel_offset: int  # ps
    sync_channel_enable: bool
    inputs: list[DeviceInputChannelConfig]


class Device:
    device_index: int
    is_open: bool
    config: DeviceConfig

    def __init__(self, index: int, config: DeviceConfig) -> None:
        self.device_index = index
        self.is_open = False
        self.config = config
        self.oflcorrection = 0

    def __open(self):
        if self.is_open:
            return
        dev_id = self.device_index
        mh.open_device(dev_id)
        self.is_open = True
        mh.initialize(dev_id, mh.Mode.T2, mh.RefSource.InternalClock)
        num_inputs = mh.get_number_of_input_channels(dev_id)
        c = self.config

        if num_inputs != len(self.config["inputs"]):
            print(
                "warning: num_inputs != len(config.inputs)",
                num_inputs,
                len(self.config["inputs"]),
            )

        mh.set_sync_divider(dev_id, c["sync_divider"])
        mh.set_sync_edge_trigger(dev_id, c["sync_edge_trigger_level"], c["sync_edge"])
        mh.set_sync_channel_offset(dev_id, 0)
        mh.set_sync_channel_enable(dev_id, c["sync_channel_enable"])
        for ch in range(0, num_inputs):
            ch_config = self.config["inputs"][ch]
            mh.set_input_edge_trigger(
                dev_id, ch, ch_config["edge_trigger_level"], ch_config["edge_trigger"]
            )
            mh.set_input_channel_offset(dev_id, ch, ch_config["channel_offset"])
            mh.set_input_channel_enable(dev_id, ch, ch_config["enable"])

    def __close(self):
        mh.close_device(self.device_index)
        self.is_open = False

    def open(self) -> "Device":
        self.__open()
        return self

    def __enter__(self):
        self.__open()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.__close()

    def start_measurement(self, meas_time: int):
        dev_id = self.device_index
        progress = 0
        self.oflcorrection = 0
        channels = []
        timestamps = []
        filename = f".arrows/{int(time.time())}-{meas_time}.arrow"
        mh.start_measurement(dev_id, meas_time)
        batches = []
        while True:
            flags = mh.get_flags(dev_id)
            if flags & 2:
                print("fifo overrun")
                mh.stop_measurement(dev_id)

            num_records, data = mh.read_fifo(dev_id)
            if num_records > 0:
                for i in range(0, num_records):
                    result = self.parse_record(data[i])
                    if result is not None:
                        ch, timestamp = result
                        channels.append(ch)
                        timestamps.append(timestamp)
                if len(channels) > 10000:
                    batches.append(
                        pa.RecordBatch.from_arrays(
                            [
                                pa.array(channels, type=pa.int8()),
                                pa.array(timestamps, type=pa.float64()),
                            ],
                            ["ch", "timestamp"],
                        )
                    )
                    channels = []
                    timestamps = []
                progress += num_records
                sys.stdout.write("\rProgress:%9u" % progress)
                sys.stdout.flush()
            else:
                status = mh.ctc_status(dev_id)
                if status > 0:
                    print("done")
                    break

        mh.stop_measurement(dev_id)
        with pa.ipc.new_file(filename, schema=TimeTagDataSchema) as f:
            for batch in batches:
                f.write_batch(batch)
        return filename

    def parse_record(self, data: int) -> tuple[int, float] | None:
        special = (data >> 31) & 0x01  # 最上位ビット
        channel = (data >> 25) & 0x3F  # 次の6ビット
        timetag = data & 0x1FFFFFF
        if special == 1:
            if channel == 0x3F:  # Overflow
                if timetag == 0:  # old style overflow, shouldn't happen
                    self.oflcorrection += T2WRAPAROUND_V2
                else:
                    self.oflcorrection += T2WRAPAROUND_V2 * timetag
            # if channel >= 1 and channel <= 15: # markers
            #     truetime = oflcorrection + timetag
            if channel == 0:  # sync
                truetime = self.oflcorrection + timetag
                return (channel, truetime * 0.2)
        else:  # regular input channel
            truetime = self.oflcorrection + timetag
            return (channel, truetime * 0.2)


def list_device_index():
    available_devices = []
    for i in range(0, 8):
        try:
            mh.open_device(i)
            available_devices.append(i)
        except Exception as e:
            pass

    for i in range(0, 8):
        mh.close_device(i)
    return available_devices


if __name__ == "__main__":
    dev_ids = list_device_index()
    if not dev_ids:
        print("no device")
        # exit(0)
    print("available devices: ", dev_ids)
    config: DeviceConfig = {
        "sync_channel_offset": 0,
        "sync_divider": 1,
        "sync_edge": mh.Edge.Falling,
        "sync_edge_trigger_level": -70,
        "sync_channel_enable": True,
        "inputs": [
            {
                "enable": True,
                "channel_offset": 0,
                "edge_trigger": mh.Edge.Falling,
                "edge_trigger_level": -70,
            }
        ]
        * 16,
    }
    dev = Device(dev_ids[0], config)
    with dev.open() as d:
        fname = dev.start_measurement(1000)
        print(fname)
