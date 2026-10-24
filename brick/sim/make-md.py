"""A tiny Mega Drive ROM for the simulator, made from nothing: it turns the screen red in the
320-wide mode and waits, so a scenario can insert, play and eject a real cart under Genesis Plus
GX without shipping anyone's game. Usage: python3 make-md.py OUT.md"""
import struct
import sys

rom = bytearray(0x8000)
rom[0:8] = struct.pack('>II', 0x00FFFE00, 0x00000200)  # stack, entry
rom[0x100:0x110] = b'SEGA MEGA DRIVE '
rom[0x150:0x160] = b'BLAST TEST      '
rom[0x190] = ord('J')  # a 3-button pad
rom[0x1A4:0x1A8] = struct.pack('>I', len(rom) - 1)  # ROM end
rom[0x1F0:0x1F3] = b'JUE'  # names every region, so the cart goes by the name: Western
program = [
    0x41F9, 0x00C0, 0x0004,  # lea $c00004,a0: VDP control
    0x43F9, 0x00C0, 0x0000,  # lea $c00000,a1: VDP data
    0x30BC, 0x8004,          # full palette (without it colours keep only their lowest bit)
    0x30BC, 0x8104,          # mode 5, display still off: registers above 10 need it first
    0x30BC, 0x8C81,          # 320 wide
    0x30BC, 0x8F02,          # auto-increment 2
    0x30BC, 0x8700,          # the backdrop is colour 0
    0x20BC, 0xC000, 0x0000,  # write colour 0 ...
    0x32BC, 0x000E,          # ... red
    0x30BC, 0x8144,          # display on
    0x60FE,                  # bra.s to itself
]
rom[0x200:0x200 + 2 * len(program)] = struct.pack(f'>{len(program)}H', *program)
with open(sys.argv[1], 'wb') as f:
    f.write(rom)
