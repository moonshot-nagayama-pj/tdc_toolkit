class InvalidStateException(Exception):
    """Thrown when a method is called on an object, but the object is
    not in an appropriate state for that function to be called.

    For example, if an object processes streams of data, and those
    streams have already been closed, it should not be possible to
    re-open them.

    """


class FifoOverrunException(Exception):
    """Thrown during measurement when the MultiHarp's buffer has
    filled and continuing the measurement is not possible.

    """


class MeasurementCompletedException(Exception):
    """Thrown during measurement when the requested time for
    measurment has elapsed and the device will no longer return
    further measurements.

    """
