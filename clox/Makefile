# This Makefile is largely based on
# https://github.com/munificent/craftinginterpreters/blob/6c2ea6f/util/c.make
.PHONY: clean run valgrind

CFLAGS = -std=c99 -Ibuild
CFLAGS += -Wall -Wextra -Werror -pedantic -pedantic-errors
CFLAGS += -O3 -flto

BUILD_DIR = build
HEADERS := $(wildcard *.h)
SOURCES := $(wildcard *.c)
OBJECTS := $(addprefix $(BUILD_DIR)/, $(notdir $(SOURCES:.c=.o)))

$(BUILD_DIR)/clox: $(OBJECTS)
	$(CC) $(CFLAGS) -o $@ $^

# The build directory is an order-only prerequisite. See:
# https://www.gnu.org/software/make/manual/html_node/Prerequisite-Types.html
$(BUILD_DIR)/%.o: %.c $(HEADERS) | $(BUILD_DIR)
	$(CC) -c $(CFLAGS) -o $@ $<

$(BUILD_DIR):
	mkdir $(BUILD_DIR)

clean:
	$(RM) -r build

run: $(BUILD_DIR)/clox
	$<

valgrind: $(BUILD_DIR)/clox
	valgrind $<
