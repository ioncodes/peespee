import os
import struct
import tempfile

import idaapi

MAGIC = b"\0PBP"
HEADER_SIZE = 0x28
EXECUTABLE_OFFSET = 0x20


def accept_file(li, filename):
    li.seek(0)
    if li.read(len(MAGIC)) != MAGIC:
        return 0

    return {"format": "PSP PBP archive", "options": 1 | idaapi.ACCEPT_ARCHIVE}


def process_archive(li, archive, defmember, neflags, formatname):
    li.seek(0)
    header = li.read(HEADER_SIZE)
    if len(header) != HEADER_SIZE:
        return "PBP header is truncated"

    start, end = struct.unpack_from("<2I", header, EXECUTABLE_OFFSET)
    if start == end:
        return "PBP contains no executable"

    li.seek(start)
    executable = li.read(end - start)
    if len(executable) != end - start:
        return "PBP executable is truncated"

    handle, temp_file = tempfile.mkstemp()
    os.write(handle, executable)
    os.close(handle)

    module_name = os.path.splitext(archive)[0] + ".elf"
    return {"temp_file": temp_file, "module_name": module_name}
