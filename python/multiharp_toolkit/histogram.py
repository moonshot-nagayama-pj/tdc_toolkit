import polars as pl
from multiharp_toolkit.util_types import Channel, TimeTag


class Histogram:
    channels: list[Channel]
    bin_width: float  # ps
    name: str
    arrow_file_path: str
    hist_arrow_file_path: str

    # state
    base_ch: Channel
    base_start: TimeTag
    last_truetime: TimeTag
    df: pl.DataFrame | None

    def __init__(self, base_ch: Channel, channels: list[Channel]) -> None:
        self.base_ch = base_ch
        self.channels = channels
        self.base_start = 0
        self.last_truetime = 0
        self.name = ""
        self.df = None

    def __repr__(self) -> str:
        return f"Hist({self.base_ch}-{self.channels}, {self.name})"

    def process(self, df: pl.DataFrame) -> None:
        _df = df.with_columns(
            [
                pl.col("ch").shift(-1).alias("next_ch"),
                pl.col("timestamp").shift(-1).alias("next_timestamp"),
            ]
        ).drop_nulls()
        bin_count = 1000
        min_timediff = 0
        max_timediff = 1500  # 1.5ns
        bin_width = (max_timediff - min_timediff) / bin_count  # 1ps

        all_hist_df = pl.DataFrame({"bin": pl.Series([], dtype=pl.Float64)})
        for channel_to in self.channels:
            time_diffs = (
                _df.filter(
                    (pl.col("ch") == self.base_ch) & (pl.col("next_ch") == channel_to)
                )
                .with_columns(
                    [
                        (pl.col("next_timestamp") - pl.col("timestamp")).alias(
                            "time_diff"
                        )
                    ]
                )
                .filter(
                    (pl.col("time_diff") > min_timediff)
                    & (pl.col("time_diff") < max_timediff)
                )
                .select(["ch", "next_ch", "time_diff"])
            )

            hist_df = (
                time_diffs.with_columns(
                    [
                        ((pl.col("time_diff") - min_timediff) / bin_width)
                        .floor()
                        .alias("bin")
                    ]
                )
                .group_by("bin")
                .agg(pl.len().alias("count"))
                .sort("count")
                .rename({"count": f"count_{channel_to}"})
            )
            all_hist_df = all_hist_df.join(
                hist_df,
                on="bin",
                how="outer",
            )
            all_hist_df = all_hist_df.with_columns(
                pl.coalesce([all_hist_df["bin"], all_hist_df["bin_right"]]).alias("bin")
            ).drop("bin_right")
            all_hist_df = all_hist_df.drop("bin_right")
            all_hist_df = all_hist_df.fill_null(0)

        self.df = all_hist_df
