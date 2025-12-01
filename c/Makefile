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
TEST_TARGET = jacobi_test
SRCS = main.c common/jacobi_common.c semaphore/jacobi_semaphore.c barrier/jacobi_barrier.c omp/jacobi_omp.c naive/jacobi_naive.c unsafe_semaphore/jacobi_unsafe_semaphore.c unsafe_optimized/jacobi_unsafe_optimized.c
TEST_SRCS = test.c common/jacobi_common.c semaphore/jacobi_semaphore.c barrier/jacobi_barrier.c omp/jacobi_omp.c naive/jacobi_naive.c unsafe_semaphore/jacobi_unsafe_semaphore.c unsafe_optimized/jacobi_unsafe_optimized.c
OBJS = $(SRCS:.c=.o)
TEST_OBJS = $(TEST_SRCS:.c=.o)
INCLUDES = -I. -Icommon -Isemaphore -Ibarrier -Iomp -Inaive -Iunsafe_semaphore -Iunsafe_optimized

.PHONY: all clean run test

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS) -lm

$(TEST_TARGET): $(TEST_OBJS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS) -lm

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

clean:
	rm -f $(OBJS) $(TEST_OBJS) $(TARGET) $(TEST_TARGET)
	rm -f common/*.o semaphore/*.o barrier/*.o omp/*.o naive/*.o unsafe_semaphore/*.o unsafe_optimized/*.o

run: $(TARGET)
	./$(TARGET)

test: $(TEST_TARGET)
	./$(TEST_TARGET)
