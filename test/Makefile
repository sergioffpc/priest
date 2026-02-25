CROSS = riscv64-unknown-elf-

CC = $(CROSS)gcc
OBJCOPY = $(CROSS)objcopy
OBJDUMP = $(CROSS)objdump

CFLAGS = \
    -march=rv64i \
    -mabi=lp64 \
    -mcmodel=medany \
    -nostdlib \
    -nostartfiles \
    -static

LDFLAGS = -T link.ld

TARGET = firmware

all: $(TARGET).elf $(TARGET).bin $(TARGET).dump

$(TARGET).elf: start.S main.c link.ld
	$(CC) $(CFLAGS) $(LDFLAGS) start.S main.c -o $@

$(TARGET).bin: $(TARGET).elf
	$(OBJCOPY) -O binary $< $@

$(TARGET).dump: $(TARGET).elf
	$(OBJDUMP) -D $< > $@

clean:
	rm -f *.elf *.bin *.dump

