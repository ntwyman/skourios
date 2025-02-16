build_dir := build

.PHONY: all
all:

sel4_prefix := $(SEL4_INSTALL_DIR)
sel4_bin := $(SEL4_INSTALL_DIR)/bin


app:=$(build_dir)/root_task.elf
kernel:=$(build_dir)/kernel.elf

$(app): $(shell find crates/root_task/src -type f)
	cargo build --target-dir build/target --artifact-dir build -p root_task

$(kernel): $(app)
	$(sel4_bin)/sel4-kernel-loader-add-payload --loader $(sel4_bin)/sel4-kernel-loader --sel4-prefix $(sel4_prefix) --app $(app) -o $@
	
.PHONY: clean
clean:
	rm -rf $(build_dir)

.PHONY: image
image: $(app)

.PHONY: build
build: $(kernel)

.PHONY: run
run: $(kernel)
	qemu-system-aarch64 \
	-machine virt,virtualization=on -cpu cortex-a57 -m size=1G \
	-serial mon:stdio \
	-nographic \
	-kernel $(kernel)

.PHONY: debug
debug: $(kernel)
	echo "target running"
	qemu-system-aarch64 \
	-s -S \
	-machine virt,virtualization=on -cpu cortex-a57 -m size=1G \
	-serial mon:stdio \
	-nographic \
	-kernel $(kernel)



