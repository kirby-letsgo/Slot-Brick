# N64 and Neo Geo Pocket cores

The distributed shared libraries are built from these exact source revisions:

- ParaLLEl N64: https://github.com/libretro/parallel-n64/tree/38befe1e6048de4abc8a9ba7c4ddeadaf342cdfc
- Beetle NeoPop: https://github.com/libretro/beetle-ngp-libretro/tree/a50d5ac288a81f2104ddf43195a4efdd15c72227

Corresponding source archives are available by replacing `/tree/REVISION` with
`/archive/REVISION.tar.gz` at those URLs. Slot's additional patch and complete build recipe
are `brick/n64-gles.patch` and `brick/build-extra-cores.sh` in
https://github.com/kirby-letsgo/Slot-Brick . Build with `sh brick/docker-build.sh`.
Source archives are SHA-256 checked before use. The card uses the stock TrimUI gcc 7.5 /
glibc 2.25 toolchain. The GLES link stub is a build aid only and is never shipped; graphics
use the device's EGL/GLES2 driver.

ParaLLEl N64 includes Mupen64Plus, Rice (Rice1964 and contributors), glN64 / gles2n64,
Angrylion and libretro-common. See the accompanying GPL-2.0 text and component notices,
the LGPL-3.0 text for gles2n64, the libretro-common notice,
and the copyright and permission notices retained in the source files. Beetle NeoPop
includes NeoPop (neopop_uk), Mednafen and libretro contributions under GPL-2.0-or-later.
The original sources contain the complete contributor and component copyright notices.
