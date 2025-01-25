## TODOs:
``` c
_init, _fini

#ifdef QSPI2
    #undef FLASH_USER_CODE_START_ADDR
    #define FLASH_USER_CODE_START_ADDR 0x64020000
#endif
```