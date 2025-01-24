from asyncio import Queue, QueueShutDown, TaskGroup

import structlog
from multiharp_toolkit.interface import RawRecords
from multiharp_toolkit.stub_device import StubMultiharpDevice
from multiharp_toolkit.tttr_record import T2Record, T2RecordQueueProcessor
from multiharp_toolkit.units import mhtk_ureg

log = structlog.get_logger()


async def read_queued_messages(processed_queue: Queue[T2Record]) -> None:
    try:
        while True:
            record = await processed_queue.get()
            await log.adebug(event="Read queue message", tttr_record=record)
    except QueueShutDown:
        await log.adebug("QueueShutDown received")


async def test_no_overflow() -> None:
    raw_queue: Queue[RawRecords] = Queue()
    processed_queue: Queue[T2Record] = Queue()
    mh = StubMultiharpDevice()
    queue_processor = T2RecordQueueProcessor(
        input_queue=raw_queue, output_queue=processed_queue
    )
    async with TaskGroup() as tg:
        mh_task = tg.create_task(
            mh.stream_measurement(1000 * mhtk_ureg.millisecond, raw_queue)
        )
        processor_task = tg.create_task(queue_processor.open())
        printer_task = tg.create_task(read_queued_messages(processed_queue))
    await log.adebug(
        event="Task results",
        mh_task=mh_task.result(),
        processor_task=processor_task.result(),
        printer_task=printer_task.result(),
    )
