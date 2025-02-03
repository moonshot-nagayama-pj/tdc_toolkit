import asyncio
import datetime
import os
import tempfile
from asyncio import QueueShutDown
from pathlib import Path

import pyarrow
from multiharp_toolkit.tttr_record import T2Record
from pyarrow import RecordBatch
from pyarrow.parquet import ParquetWriter


async def write_parquet(
    input_queue: asyncio.Queue[T2Record],
    output_dir: os.PathLike[str],
    name: str = "measurement",
    chunk_rows: int = 2900000,
    file_rows: int = 29000000,
) -> None:
    """Write a series of Parquet files to disk containing the data
    from the input queue.

    For write efficiency and ease in handling large volumes of data,
    we batch writes to Parquet files in chunks of about 200 MiB (as
    recommended in `this discussion
    <https://github.com/apache/arrow/issues/13142>`_), and then rotate
    to a new file approximately every 2 GiB. Rows are assumed to
    contain about 72 bits of data each; ignoring metadata overhead and
    compression, this means that a 2 GiB file can hold approximately
    29,826,161 rows. For simplicity, we set the default size limit for
    each file to 29,000,000 rows, and default chunk size to 2,900,000.

    The GitHub issue above discusses using ``sys.getsizeof()`` to
    determine the number of bytes used by a data structure. This may
    work for some object types, but, generally speaking,
    ``getsizeof()`` will only report the amount of memory used by a
    container object, not the objects contained within that
    object. For this reason, we rely on row counts rather than actual
    byte counts.

    """
    if not Path.is_dir(Path(output_dir)):
        raise FileNotFoundError(
            f"The output directory must exist and be a directory. output_dir was {output_dir}"
        )

    schema = pyarrow.schema(
        (("channel", pyarrow.uint8()), ("time_tag", pyarrow.uint64()))
    )
    max_chunk_count = file_rows / chunk_rows
    file_timestamp = datetime.datetime.now(datetime.UTC).strftime("%Y%m%dT%H%M%SZ")
    queue_shut_down = False
    while not queue_shut_down:
        with tempfile.NamedTemporaryFile(
            dir=output_dir,
            prefix=f"{file_timestamp}_{name}_",
            suffix=".parquet",
            delete=False,
        ) as output_file:
            with ParquetWriter(output_file, schema) as writer:
                batch_dict: dict[str, list[int]] = {"channel": [], "time_tag": []}
                chunk_count = 0
                while True:
                    try:
                        record = await input_queue.get()
                    except QueueShutDown:
                        if len(batch_dict["channel"]) > 0:
                            # Write out the current batch to the
                            # current file and break the loop
                            batch = RecordBatch.from_pydict(batch_dict, schema=schema)
                            writer.write_batch(batch)
                            queue_shut_down = True
                            break

                    batch_dict["channel"].append(record[0])
                    batch_dict["time_tag"].append(record[1])
                    input_queue.task_done()
                    if len(batch_dict["channel"]) == chunk_rows:
                        # Write a chunk to disk
                        batch = RecordBatch.from_pydict(batch_dict, schema=schema)
                        writer.write_batch(batch)
                        chunk_count += 1
                        batch_dict = {"channel": [], "time_tag": []}
                        if chunk_count >= max_chunk_count:
                            # Close this file and open a new one
                            break
