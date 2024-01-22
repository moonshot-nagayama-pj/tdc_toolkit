from os import path
import os, sys, json, argparse
from multiharp_toolkit.ptu_parser import parse
from multiharp_toolkit.coincidence_counter import CoincidenceCounter, ChannelInfo
import polars as pl
import plotly.express as px


def calculate_time_diff(df, channel_from, channel_to):
    _df = df.with_columns(
        [
            pl.col("ch").shift(-1).alias("next_ch"),
            pl.col("timestamp").shift(-1).alias("next_timestamp"),
            pl.col("ch").shift(-2).alias("next_next_ch"),
            pl.col("timestamp").shift(-2).alias("next_next_timestamp"),
        ]
    ).drop_nulls()

    # 時間差分を計算
    time_diffs = (
        _df.filter((pl.col("ch") == channel_from) & (pl.col("next_ch") == channel_to))
        .with_columns(
            [(pl.col("next_timestamp") - pl.col("timestamp")).alias("time_diff")]
        )
        .filter((pl.col("time_diff") > 0) & (pl.col("time_diff") < 10000))
        .select(["ch", "next_ch", "time_diff"])
    )

    return time_diffs


def extract_peak(df, channel_from, channel_to, peak_width):
    _df = calculate_time_diff(df, channel_from, channel_to)
    # ビンの範囲と数を定義
    bin_count = 1000
    min_timediff = _df["time_diff"].min()
    max_timediff = _df["time_diff"].max()
    bin_width = (max_timediff - min_timediff) / bin_count

    hist_df = (
        _df.with_columns(
            [((pl.col("time_diff") - min_timediff) / bin_width).floor().alias("bin")]
        )
        .group_by("bin")
        .agg(pl.count().alias("count"))
        .sort("count", descending=True)
    )

    peak_bin = hist_df[0]

    peak = peak_bin["bin"] * bin_width + min_timediff + bin_width * 0.5
    print("peak", channel_to, ":", peak[0], "ps")
    return (peak[0] - peak_width, peak[0] + peak_width)


def plot_timediff_hist(df, filename):
    diff01_df = calculate_time_diff(df, 0, 1)
    diff02_df = calculate_time_diff(df, 0, 2)
    diff0102_df = pl.concat(
        [
            diff01_df.with_columns([pl.col("time_diff").alias("time_diff1")])
            .filter(pl.col("time_diff1") < 1500)
            .select("time_diff1"),
            diff02_df.with_columns([pl.col("time_diff").alias("time_diff2")])
            .filter(pl.col("time_diff2") < 1500)
            .select("time_diff2"),
        ],
        how="horizontal",
    )
    fig = px.histogram(
        diff0102_df.to_pandas(), x=["time_diff1", "time_diff2"], nbins=int(10000)
    )
    fig.update_layout(bargap=0.2)
    fig.write_image(filename + ".png")
    fig.write_html(filename + ".html")


def calc_g2(df, peak_start_1, peak_end_1, peak_start_2, peak_end_2):
    num_records = len(df["ch"])
    df_ch = df["ch"].to_list()
    df_timestamp = df["timestamp"].to_list()

    ch_sync = ChannelInfo(0)
    ch_1 = ChannelInfo(1, peak_start_1, peak_end_1)
    ch_2 = ChannelInfo(2, peak_start_2, peak_end_2)
    counter = CoincidenceCounter(
        coincidence_targets=[
            [ch_sync, ch_1, ch_2],
            [ch_sync, ch_2],
            [ch_sync, ch_1],
        ]
    )
    for i, ch in enumerate(df_ch):
        timestamp = df_timestamp[i]
        counter.process(ch, timestamp)
        if i % 100000 == 0:
            sys.stdout.write(
                "\rCount events...: %.1f%%" % (float(i) * 100 / float(num_records))
            )
            sys.stdout.flush()
    n_sync = counter.number_of_counts[0]
    n_sync_1 = counter.coincidence_counts["[0, 1]"]
    n_sync_2 = counter.coincidence_counts["[0, 2]"]
    n_sync_1_2 = counter.coincidence_counts["[0, 1, 2]"]

    print(
        "\n",
        dict(
            n_sync=n_sync, n_sync_1=n_sync_1, n_sync_2=n_sync_2, n_sync_1_2=n_sync_1_2
        ),
    )
    print(f"n_sync_1 / n_sync: {n_sync_1 / n_sync}")
    print(f"n_sync_2 / n_sync: {n_sync_2 / n_sync}")
    print("g2:", (n_sync * n_sync_1_2) / (n_sync_1 * n_sync_2))
    return {
        "peak_start_1": peak_start_1,
        "peak_end_1": peak_end_1,
        "peak_start_2": peak_start_2,
        "peak_end_2": peak_end_2,
        "n_sync": n_sync,
        "n_sync_1": n_sync_1,
        "n_sync_2": n_sync_2,
        "n_sync_1_2": n_sync_1_2,
        "n_sync_1/n_sync": n_sync_1 / n_sync,
        "n_sync_2/n_sync": n_sync_2 / n_sync,
        "g2": (n_sync * n_sync_1_2) / (n_sync_1 * n_sync_2),
    }


def main():
    parser = argparse.ArgumentParser(
        description="calculate a g^(2) value. if you don't provide peak values, it is caluculated automatically"
    )

    parser.add_argument(
        "inputfile",
        type=str,
        help="Path to the ptu file for T2 Mode measurement result",
    )

    parser.add_argument(
        "-o",
        "--output",
        type=str,
        help="Path to put the result files",
        default="./result",
        required=False,
    )
    parser.add_argument("--peak1", type=float, help="peak 1 value (ps)", required=False)
    parser.add_argument("--peak2", type=float, help="peak 2 value (ps)", required=False)
    parser.add_argument(
        "--peak-width", type=float, help="peak width (ps)", default=50, required=False
    )

    # Parse the arguments
    args = parser.parse_args()
    filepath = args.inputfile
    resultdir = args.output
    if not path.exists(resultdir):
        print("result dir does not exist. make the dir", resultdir)
        os.mkdir(resultdir)
    result_file_name = path.join(
        resultdir, path.basename(filepath)[0:-4] + "_result.json"
    )
    image_file_name = path.join(resultdir, path.basename(filepath)[0:-4] + "_hist")
    peak_width = args.peak_width
    peak1 = args.peak1
    peak2 = args.peak2
    channel_swapped = False
    print("loading file: ", filepath)
    print("result: ", resultdir)
    print("result json: ", result_file_name)
    with open(filepath, "rb") as f:
        result = parse(f)
    assert result is not None, "Failed to parse"

    print(
        "\nevent counts: ",
        [f"ch{i}: {len(ch)}" for i, ch in enumerate(result.events) if len(ch) > 0],
    )

    data = result.events[0] + result.events[1] + result.events[2]
    data.sort()
    df = pl.concat(
        [
            pl.DataFrame({"timestamp": result.events[0], "ch": 0}),
            pl.DataFrame({"timestamp": result.events[1], "ch": 1}),
            pl.DataFrame({"timestamp": result.events[2], "ch": 2}),
        ]
    ).sort("timestamp")

    if peak1 and peak2:
        peak_start_1 = peak1 - peak_width
        peak_end_1 = peak1 + peak_width
        peak_start_2 = peak2 - peak_width
        peak_end_2 = peak2 + peak_width
    else:
        print("peak values are not provided. calculate peak values")
        peak_start_1, peak_end_1 = extract_peak(df, 0, 1, peak_width=peak_width)
        peak_start_2, peak_end_2 = extract_peak(df, 0, 2, peak_width=peak_width)

    if peak_start_1 > peak_start_2:  # swap ch1 and ch2
        channel_swapped = True
        df = pl.concat(
            [
                pl.DataFrame({"timestamp": result.events[0], "ch": 0}),
                pl.DataFrame({"timestamp": result.events[2], "ch": 1}),
                pl.DataFrame({"timestamp": result.events[1], "ch": 2}),
            ]
        ).sort("timestamp")
        e = peak_end_1
        s = peak_start_1
        peak_end_1 = peak_end_2
        peak_start_1 = peak_start_2
        peak_end_2 = e
        peak_start_2 = s
        print("channel 1 and 2 were swapped")

    print(f"peak1: {peak_start_1} ~ {peak_end_1} (ps)")
    print(f"peak2: {peak_start_2} ~ {peak_end_2} (ps)")

    res = calc_g2(df, peak_start_1, peak_end_1, peak_start_2, peak_end_2)
    if not channel_swapped:
        res["num_ch1_events"] = len(result.events[1])
        res["num_ch2_events"] = len(result.events[2])
    else:
        res["num_ch1_events"] = len(result.events[2])
        res["num_ch2_events"] = len(result.events[1])

    with open(result_file_name, "w") as f:
        json.dump(res, f, indent=2)

    print("plottting... ", image_file_name)
    plot_timediff_hist(df, image_file_name)


if __name__ == "__main__":
    print("__main__")
    main()
