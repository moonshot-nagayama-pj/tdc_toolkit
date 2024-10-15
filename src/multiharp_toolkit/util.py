import time

from types import TracebackType

class PerfCounter:
    start_time: float
    name: str | None

    def __init__(self, name: str | None = None) -> None:
        self.start_time = 0
        self.name = name

    def __enter__(self) -> None:
        self.start_time = time.perf_counter()

    def __exit__(self, exc_type: type[BaseException] | None, exc_value: BaseException | None, exc_tb: TracebackType | None) -> None:
        elapsed_time = time.perf_counter() - self.start_time
        if self.name is not None:
            print(f"[{self.name}]elapsed_time: {elapsed_time:.2f}sec")
        else:
            print(f"elapsed_time: {elapsed_time:.2f}sec")
