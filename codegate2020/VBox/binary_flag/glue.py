#!/usr/bin/python3

import os
import sys
import base64
import struct
import tempfile


PWD = os.path.dirname(os.path.realpath(__file__))


def write(x):
    sys.stdout.write(x)
    sys.stdout.flush()


def read_binary(fd):
    write('Give me your binary (base64 encoded, mark end with newline): ')
    binary = ''
    buf = ''
    while '\n' not in buf:
        binary += buf
        buf = sys.stdin.readline()
    binary += buf.split('\n')[0]
    try:
        os.write(fd, base64.b64decode(binary))
        os.close(fd)
    except Exception as e:
        write("Invalid input.: {}\n".format(e))
        exit(-1)


def launch(binary):
    os.execve('/home/vbox/vbox', ['vbox', binary], os.environ)
    write('Fail to launch vbox.\n')
    exit(-1)


def main():
    (fd, name) = tempfile.mkstemp()
    read_binary(fd)
    launch(name)


if __name__ == '__main__':
    exit(main())
