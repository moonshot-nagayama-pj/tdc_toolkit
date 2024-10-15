# based on MultiHarp150_160_V3_1 from https://www.picoquant.com/products/category/tcspc-and-time-tagging-modules/multiharp-160#custom1
#
# Read_PTU.py    Read PicoQuant Unified Histogram Files
# This is demo code. Use at your own risk. No warranties.
# Keno Goertz, PicoQUant GmbH, February 2018

import io
import struct
import sys
import time
from typing import Any

# Tag Types
ty_empty_8 = struct.unpack(">i", bytes.fromhex("FFFF0008"))[0]
ty_bool_8 = struct.unpack(">i", bytes.fromhex("00000008"))[0]
ty_int_8 = struct.unpack(">i", bytes.fromhex("10000008"))[0]
ty_bit_set_64 = struct.unpack(">i", bytes.fromhex("11000008"))[0]
ty_color_8 = struct.unpack(">i", bytes.fromhex("12000008"))[0]
ty_float_8 = struct.unpack(">i", bytes.fromhex("20000008"))[0]
ty_t_date_time = struct.unpack(">i", bytes.fromhex("21000008"))[0]
ty_float_8_array = struct.unpack(">i", bytes.fromhex("2001FFFF"))[0]
ty_ansi_string = struct.unpack(">i", bytes.fromhex("4001FFFF"))[0]
ty_wide_string = struct.unpack(">i", bytes.fromhex("4002FFFF"))[0]
ty_binary_blob = struct.unpack(">i", bytes.fromhex("FFFFFFFF"))[0]


class TimeTaggedData:
    names: list[str]
    values: list[Any]
    num_records: int
    glob_res: float
    events: list[list[int | float]]  # [channel: [timetag]]


class Parser:
    events: list[list[int | float]]  # [channel: [timetag]]
    timestamps: list[float]  # for combined channel mode
    channels: list[int]  # for combined channel mode
    oflcorrection: int
    ptu_version: int
    T2WRAPAROUND_V1 = 33552000
    T2WRAPAROUND_V2 = 33554432
    combined_channel: bool

    def __init__(self, ptu_version: int = 2) -> None:
        self.oflcorrection = 0
        self.ptu_version = ptu_version
        self.events = [[] for i in range(0, 65)]  # max 64ch
        self.channels = []
        self.timestamps = []
        self.combined_channel = False
        self.time_resolution = 5

    def __repr__(self) -> str:
        num_ev_str = ",".join(
            [
                f"{ch}:{len(events)}events"
                for ch, events in enumerate(self.events)
                if len(events) > 0
            ]
        )
        return f"Parser(events: {num_ev_str}, v{self.ptu_version}, ofl: {self.oflcorrection})"

    def reset(self) -> None:
        self.oflcorrection = 0
        self.channels = []
        self.timestamps = []
        self.events = [[] for i in range(0, 65)]  # max 64ch

    def parse_records(self, inputfile: io.BufferedReader, num_records: int) -> None:
        for i in range(0, num_records):
            data = struct.unpack("<I", inputfile.read(4))[0]
            self.parse_record(data)
            if i % 100000 == 0:
                sys.stdout.write(
                    f"\rLoading file: {(float(i) * 100 / float(num_records)):.1f}%"
                )
                sys.stdout.flush()

    def parse_record(self, data: int) -> None:
        special = (data >> 31) & 0x01  # 最上位ビット
        channel = (data >> 25) & 0x3F  # 次の6ビット
        timetag = data & 0x1FFFFFF
        if special == 1:
            if channel == 0x3F:  # Overflow
                # Number of overflows in nsync. If old version, it's an
                # old style single overflow
                if self.ptu_version == 1:
                    self.oflcorrection += Parser.T2WRAPAROUND_V1
                else:
                    if timetag == 0:  # old style overflow, shouldn't happen
                        self.oflcorrection += Parser.T2WRAPAROUND_V2
                    else:
                        self.oflcorrection += Parser.T2WRAPAROUND_V2 * timetag
            # if channel >= 1 and channel <= 15: # markers
            #     truetime = oflcorrection + timetag
            if channel == 0:  # sync
                truetime = self.oflcorrection + timetag
                self.append_events(0, truetime)
        else:  # regular input channel
            truetime = self.oflcorrection + timetag
            self.append_events(channel + 1, truetime)

    def convert_timetag_to_relative_timestamp(self, timetag: int) -> int:
        """convert time tag to time(unit: psec)"""
        return timetag * self.time_resolution

    def append_events(self, channel: int, timestamp: int) -> None:
        if self.combined_channel:
            self.channels.append(channel)
            self.timestamps.append(
                self.convert_timetag_to_relative_timestamp(timestamp)
            )
        else:
            self.events[channel].append(
                self.convert_timetag_to_relative_timestamp(timestamp)
            )


def parse_header(inputfile: io.BufferedReader) -> tuple[list[str], list[Any]] | None:
    # pylint: disable=too-many-branches,too-many-statements
    magic = inputfile.read(8).decode("utf-8").strip("\0")
    if magic != "PQTTTR":
        print("ERROR: Magic invalid, this is not a PTU file.")
        return None

    # read the version string e.g. 1.1.02
    inputfile.read(8).decode("utf-8").strip("\0")
    # Write the header data to outputfile and also save it in memory.
    # There's no do ... while in Python, so an if statement inside the while loop
    # breaks out of it
    tag_data_list: list[tuple[str, Any]] = []  # Contains tuples of (tagName, tagValue)
    while True:
        tag_ident = inputfile.read(32).decode("utf-8").strip("\0")
        tag_idx = struct.unpack("<i", inputfile.read(4))[0]
        tag_typ = struct.unpack("<i", inputfile.read(4))[0]
        if tag_idx > -1:
            eval_name = tag_ident + "(" + str(tag_idx) + ")"
        else:
            eval_name = tag_ident
        # outputfile.write("\n%-40s" % eval_name)
        if tag_typ == ty_empty_8:
            inputfile.read(8)
            # outputfile.write("<empty Tag>")
            tag_data_list.append((eval_name, "<empty Tag>"))
        elif tag_typ == ty_bool_8:
            tag_int = struct.unpack("<q", inputfile.read(8))[0]
            if tag_int == 0:
                # outputfile.write("False")
                tag_data_list.append((eval_name, "False"))
            else:
                # outputfile.write("True")
                tag_data_list.append((eval_name, "True"))
        elif tag_typ == ty_int_8:
            tag_int = struct.unpack("<q", inputfile.read(8))[0]
            # outputfile.write("%d" % tag_int)
            tag_data_list.append((eval_name, tag_int))
        elif tag_typ == ty_bit_set_64:
            tag_int = struct.unpack("<q", inputfile.read(8))[0]
            # outputfile.write("{0:#0{1}x}".format(tag_int,18))
            tag_data_list.append((eval_name, tag_int))
        elif tag_typ == ty_color_8:
            tag_int = struct.unpack("<q", inputfile.read(8))[0]
            # outputfile.write("{0:#0{1}x}".format(tag_int,18))
            tag_data_list.append((eval_name, tag_int))
        elif tag_typ == ty_float_8:
            tag_float = struct.unpack("<d", inputfile.read(8))[0]
            # outputfile.write("%-3E" % tag_float)
            tag_data_list.append((eval_name, tag_float))
        elif tag_typ == ty_float_8_array:
            tag_int = struct.unpack("<q", inputfile.read(8))[0]
            # outputfile.write("<Float array with %d entries>" % tag_int/8)
            tag_data_list.append((eval_name, tag_int))
        elif tag_typ == ty_t_date_time:
            tag_float = struct.unpack("<d", inputfile.read(8))[0]
            tag_time_int = int((tag_float - 25569) * 86400)
            tag_time = time.gmtime(tag_time_int)
            # outputfile.write(time.strftime("%a %b %d %H:%M:%S %Y", tag_time))
            tag_data_list.append((eval_name, tag_time))
        elif tag_typ == ty_ansi_string:
            tag_int = struct.unpack("<q", inputfile.read(8))[0]
            tag_string = inputfile.read(tag_int).decode("utf-8").strip("\0")
            # outputfile.write("%s" % tag_string)
            tag_data_list.append((eval_name, tag_string))
        elif tag_typ == ty_wide_string:
            tag_int = struct.unpack("<q", inputfile.read(8))[0]
            tag_string = (
                inputfile.read(tag_int).decode("utf-16le", errors="ignore").strip("\0")
            )
            # outputfile.write(tag_string)
            tag_data_list.append((eval_name, tag_string))
        elif tag_typ == ty_binary_blob:
            tag_int = struct.unpack("<q", inputfile.read(8))[0]
            # outputfile.write("<Binary blob with %d bytes>" % tag_int)
            tag_data_list.append((eval_name, tag_int))
        else:
            print("ERROR: Unknown tag type", tag_typ)
            sys.exit(0)
        if tag_ident == "Header_End":
            break

    # Reformat the saved data for easier access
    return [tag_data_list[i][0] for i in range(0, len(tag_data_list))], [
        tag_data_list[i][1] for i in range(0, len(tag_data_list))
    ]


def parse(inputfile: io.BufferedReader) -> TimeTaggedData | None:
    headers = parse_header(inputfile)
    assert headers is not None, "failed to parse header"
    tag_names, tag_values = headers
    ret = TimeTaggedData()
    ret.names = tag_names
    ret.values = tag_values
    ret.events = [[] for i in range(0, 65)]

    # get important variables from headers
    ret.num_records = tag_values[tag_names.index("TTResult_NumberOfRecords")]
    ret.glob_res = tag_values[tag_names.index("MeasDesc_GlobalResolution")]
    print({"glob_res": ret.glob_res, "num_records": ret.num_records})

    ctx = Parser(ptu_version=2)
    ctx.parse_records(inputfile, ret.num_records)
    ret.events = ctx.events
    return ret
