TARGET := orm
SRCS := src/queue.c src/run.c src/main.c src/helper.c src/loader.c src/region.c
SRCS += src/devs/device.c src/devs/tty.c
CFLAGS := -Wall -I./inc -std=gnu99 -O0 -Wno-format-extra-args -lpthread
CC := gcc

ifndef DEPLOY
	CFLAGS += -DDEBUG
endif

all: clean $(TARGET)

$(TARGET): $(SRCS) $(DEPS)
	$(CC) $^ $(CFLAGS) -o $@

deploy:
	$(MAKE) DEPLOY=1

clean:
	$(RM) $(TARGET)

run:
	./orm ./orm-compiler/a.out
