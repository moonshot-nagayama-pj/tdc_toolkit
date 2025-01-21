import sys
from types import TracebackType
from typing import Any

import structlog

from python_tests.logs import setup_log

setup_log("tests")
log = structlog.get_logger()


def excepthook(
    exception_type: type[BaseException],
    e: BaseException,
    traceback: TracebackType | None,
) -> Any:
    log.error(event="uncaught_exception", exc_info=e)
    return sys.__excepthook__(exception_type, e, traceback)


sys.excepthook = excepthook
