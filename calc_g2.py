from mh_file_parser import parse
import polars as pl
import plotly.express as px

inputfile = open("./sampledata/default_pulse.ptu", "rb")
# inputfile = open("./sampledata/default_laser.ptu", "rb")
result = parse(inputfile)

print(
    "\nevent counts : ",
    [f"ch{i}: {len(ch)}" for i, ch in enumerate(result.events) if len(ch) > 0],
)

data = result.events[0] + result.events[1] + result.events[2]
data.sort()
df = (
    pl.concat(
        [
            pl.DataFrame({"timestamp": result.events[0], "ch": 0}),
            pl.DataFrame(
                {"timestamp": result.events[1], "ch": 2}
            ),  # switch ch 1 and 2 for arrival time order
            pl.DataFrame({"timestamp": result.events[2], "ch": 1}),
        ]
    )
    .sort("timestamp")
    .with_columns(
        [
            pl.col("ch").shift(-1).alias("next_ch"),
            pl.col("timestamp").shift(-1).alias("next_timestamp"),
            pl.col("ch").shift(-2).alias("next_next_ch"),
            pl.col("timestamp").shift(-2).alias("next_next_timestamp"),
        ]
    )
    .drop_nulls()
)


def calculate_time_diff(df, channel_from, channel_to):
    # 時間差分を計算
    time_diffs = (
        df.filter((pl.col("ch") == channel_from) & (pl.col("next_ch") == channel_to))
        .with_columns(
            [(pl.col("next_timestamp") - pl.col("timestamp")).alias("time_diff")]
        )
        .filter((pl.col("time_diff") > 0) & (pl.col("time_diff") < 10000))
        .select(["ch", "next_ch", "time_diff"])
    )

    return time_diffs


diff01_df = calculate_time_diff(df, 0, 1)
diff02_df = calculate_time_diff(df, 0, 2)


def extract_peak(df):
    # ビンの範囲と数を定義
    bin_count = 1000
    min_timediff = df["time_diff"].min()
    max_timediff = df["time_diff"].max()
    bin_width = (max_timediff - min_timediff) / bin_count

    # ビンで集計
    hist_df = (
        df.with_columns(
            [((pl.col("time_diff") - min_timediff) / bin_width).floor().alias("bin")]
        )
        .groupby("bin")
        .agg(pl.count().alias("count"))
        .sort("count", descending=True)
    )

    # ピークビンを取得
    peak_bin = hist_df[0]

    # ピーク期間を計算
    peak_start = peak_bin["bin"] * bin_width + min_timediff
    peak_end = peak_start + bin_width

    # ピーク期間を表示
    peak_width = 50
    print("ピーク期間:", peak_start[0] - peak_width, "ps から", peak_end[0] + peak_width, "ps")
    return (peak_start[0] - peak_width, peak_start[0] + peak_width)


peak_start_1, peak_end_1 = extract_peak(diff01_df)
peak_start_2, peak_end_2 = extract_peak(diff02_df)


def calc_g2():
    sync_start = 0
    ch1_found = False
    n_sync_1 = 0
    n_sync = 0
    n_sync_2 = 0
    n_sync_1_2 = 0

    num_records = len(df["ch"])
    df_ch = df["ch"].to_list()
    df_timestamp = df["timestamp"].to_list()
    import sys

    for i, ch in enumerate(df_ch):
        timestamp = df_timestamp[i]
        if ch == 0:
            sync_start = timestamp
            n_sync += 1
            ch1_found = False
            continue
        diff = timestamp - sync_start
        if ch == 1:
            if peak_start_1 < diff < peak_end_1:
                n_sync_1 += 1
                ch1_found = True
        if ch == 2:
            if peak_start_2 < diff < peak_end_2:
                n_sync_2 += 1
                if ch1_found:
                    n_sync_1_2 += 1
        if i % 100000 == 0:
            sys.stdout.write(
                "\rProgress: %.1f%%" % (float(i) * 100 / float(num_records))
            )
            sys.stdout.flush()

    print(
        dict(n_sync=n_sync, n_sync_1=n_sync_1, n_sync_2=n_sync_2, n_sync_1_2=n_sync_1_2)
    )
    print("g2:", (n_sync * n_sync_1_2) / (n_sync_1 * n_sync_2))


calc_g2()
