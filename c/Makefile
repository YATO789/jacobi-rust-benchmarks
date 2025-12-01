CC = gcc
CFLAGS = -O3 -Wall -Wextra -pthread
# macOS specific check for OpenMP (libomp)
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    # Assuming brew install libomp
    CFLAGS += -Xpreprocessor -fopenmp -I/opt/homebrew/opt/libomp/include
    LDFLAGS += -L/opt/homebrew/opt/libomp/lib -lomp
else
    CFLAGS += -fopenmp
    LDFLAGS += -lgomp
endif

TARGET = jacobi_bench
SRCS = main.c common/jacobi_common.c semaphore/jacobi_semaphore.c barrier/jacobi_barrier.c omp/jacobi_omp.c naive/jacobi_naive.c
OBJS = $(SRCS:.c=.o)
INCLUDES = -I. -Icommon -Isemaphore -Ibarrier -Iomp -Inaive

.PHONY: all clean run

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS) -lm

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

clean:
	rm -f $(OBJS) $(TARGET)
	rm -f common/*.o semaphore/*.o barrier/*.o omp/*.o naive/*.o

run: $(TARGET)
	./$(TARGET)
