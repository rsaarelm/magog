#!/usr/bin/env python

import zlib
import sys

# Maximum ratio of compressed data to uncompressed that makes it worthwhile to
# compress the data.
COMPRESS_RATIO_NEEDED = 0.75

BYTES_PER_LINE = 16

DO_COMPRESS = False

def bytes(filename):
    f = open(filename, "rb")
    result = f.read()
    f.close()
    return result

def should_compress(uncompressed, compressed):
    return DO_COMPRESS and (len(compressed) / len(uncompressed) <= COMPRESS_RATIO_NEEDED)

def emit_bytes(out, bytes):
    for i, c in enumerate(bytes):
        out.write(str(c))
        if i == len(bytes) - 1:
            out.write("\n")
        elif (i % BYTES_PER_LINE) == BYTES_PER_LINE - 1:
            out.write(",\n")
        else:
            out.write(",")

def main(filename, emitted_name=None):
    if not emitted_name:
        emitted_name = filename
    data = bytes(filename)
    compressed_data = zlib.compress(data, 9)
    compressed = should_compress(data, compressed_data)
    out = sys.stdout
    if compressed:
        data = compressed_data

    out.write('#include <util/staticfile.hpp>\n\n')
    out.write('static const unsigned char data[]{\n')
    emit_bytes(out, data)
    out.write('};\n\n')

    if compressed:
        out.write("UTIL_COMPRESSED_FILE(")
    else:
        out.write("UTIL_FILE(")
    out.write('"%s", %d, data);\n' % (emitted_name, len(data)))

if __name__ == '__main__':
    if len(sys.argv) > 2:
        # Emit specific name
        main(sys.argv[1], sys.argv[2])
    else:
        main(sys.argv[1])
