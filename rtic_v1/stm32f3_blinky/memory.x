
MEMORY
{
  /* Flash memory at 0x0800 0000 - 0x081F FFFF, range 2MB and has a size of 512kB*/
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  /* RAM begins at 0x20000000 and has a size of 128kB*/
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}