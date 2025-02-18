import time
from asyncio import Queue, QueueShutDown, TaskGroup

import structlog
from multiharp_toolkit.interface import RawRecords
from multiharp_toolkit.stub_device import StubMultiharpDevice
from multiharp_toolkit.tttr_record import T2Record, T2RecordQueueProcessor
from multiharp_toolkit.units import mhtk_ureg

log = structlog.get_logger()


async def read_queued_messages(processed_queue: Queue[T2Record]) -> None:
    """The MHLib manual estimates that, when using USB 3.0, the device
    could send as many as 80,000,000 events per second.

    """
    record_count = 0
    start_time = time.perf_counter()
    try:
        while True:
            try:
                _ = await processed_queue.get()
            except QueueShutDown:
                await log.adebug("QueueShutDown received")
                break
            record_count += 1
            processed_queue.task_done()
    finally:
        await log.adebug(
            event="Final count of processed records",
            record_count=record_count,
            processing_time=(time.perf_counter() - start_time) * mhtk_ureg.seconds,
        )


async def test_no_overflow() -> None:
    raw_queue: Queue[RawRecords] = Queue()
    processed_queue: Queue[T2Record] = Queue()
    mh = StubMultiharpDevice()
    queue_processor = T2RecordQueueProcessor(
        input_queue=raw_queue, output_queue=processed_queue
    )
    async with TaskGroup() as tg:
        mh_task = tg.create_task(mh.stream_measurement(1 * mhtk_ureg.second, raw_queue))
        processor_task = tg.create_task(queue_processor.open())
        printer_task = tg.create_task(read_queued_messages(processed_queue))
    await log.adebug(
        event="Task results",
        mh_task=mh_task.result(),
        processor_task=processor_task.result(),
        printer_task=printer_task.result(),
    )
