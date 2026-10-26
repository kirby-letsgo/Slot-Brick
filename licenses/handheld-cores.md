# WonderSwan and Lynx cores

Beetle Cygne (WonderSwan and WonderSwan Color):
https://github.com/libretro/beetle-wswan-libretro/tree/4b01295838ea89e3f1355bbe4cb5cf98aa6108cd

Beetle Lynx (based on Handy by K. Wilkins, Mednafen and libretro):
https://github.com/libretro/beetle-lynx-libretro/tree/fcdefcfb3c11d6d2e71be076a5d3df2e88ab73ed

The corresponding source archive for each is available by replacing `/tree/REVISION`
with `/archive/REVISION.tar.gz`. Slot distributes unmodified cores built with the
SHA-256-verified recipe in `brick/build-handhelds.sh`, using the stock-compatible
ARM64 GCC 7.5 / glibc 2.25 toolchain. Use `sh brick/docker-build.sh` for the full
card or `sh brick/build-handhelds.sh sim` for native cores.

See the accompanying GPL-2.0 license texts and component notices. The source
archives preserve the complete copyright and permission notices. No game or
Lynx BIOS data is distributed with Slot.
