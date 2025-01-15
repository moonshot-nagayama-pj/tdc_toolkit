class InvalidStateException(Exception):
    """Thrown when a method is called on an object, but the object is
    not in an appropriate state for that function to be called.

    For example, if an object processes streams of data, and those
    streams have already been closed, it should not be possible to
    re-open them.

    """
