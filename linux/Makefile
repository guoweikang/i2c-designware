# SPDX-License-Identifier: GPL-2.0

KDIR ?= /home/guoweikang/code/cicv/kernel/build-rust
ifndef KDIR
$(error The kernel source directory (KDIR) is not defined)
endif
ifeq ($(shell test -d $(KDIR) && echo true),)
$(error Invalid path specified in KDIR)
endif

PWD := $(shell pwd)
CARGO_MOD_DIR := /home/guoweikang/code/cicv/i2c-designware/linux/i2c-dw-plat
CARGO_MOD_TARGET_DIR := /home/guoweikang/code/cicv/i2c-designware/linux/target
CARGO_MODDEPS_DIR := /home/guoweikang/code/cicv/i2c-designware/linux/target/release/deps
CARGO_OUTPUT_DIR := /home/guoweikang/code/cicv/i2c-designware/linux

obj-m	+= i2c-dw-plat.o

default:
	echo $(CARGO_MOD_DIR)
	$(MAKE) CROSS_COMPILE=aarch64-none-linux-gnu-  ARCH=arm64 LLVM=-17 -C $(KDIR) M=$$PWD V=1
clean:
	@make -C $(KDIR) M=$(PWD) clean
	@rm -r $(CARGO_MOD_TARGET_DIR)

fmt:
	@cd $(CARGO_MOD_DIR) && cargo fmt
