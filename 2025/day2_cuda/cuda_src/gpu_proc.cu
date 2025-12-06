#include <cstdio>
#include <cstdlib>
#include <cub/cub.cuh>
#include <cuda/cmath>

__device__ void part1_inner(uint64_t *item) {
  /// div_lookup[[idx]] gives 10 to the power of idx.
  ///
  /// Contains half of the powers of ten in a uint64_t.
  /// Since we are always accessing halves, that is the highest value.
  ///
  /// Expected that on most warps, all items get the same div lookup.
  __constant__ static uint64_t div_lookup[10] = {
      1u,      10u,      100u,      1000u,      10000u,
      100000u, 1000000u, 10000000u, 100000000u, 1000000000u,
  };

  const uint32_t num_digits = cuda::ilog10(*item) + 1;
  const unsigned long long int div = div_lookup[(num_digits / 2)];

  // Placeholder
  const bool passes = (*item / div) == (*item % div);
  // Since true is 1 and false is 0, this is a nice branchless trick.
  *item *= passes;
}

__global__ void part1_512threads(const uint64_t *input, uint64_t *output) {
  using BlockReduce =
      cub::BlockReduce<unsigned long long int, 512,
                       cub::BLOCK_REDUCE_RAKING_COMMUTATIVE_ONLY>;
  __shared__ typename BlockReduce::TempStorage sum_storage;

  const int idx = (blockDim.x * blockIdx.x) + threadIdx.x;
  uint64_t item = input[idx];

  part1_inner(&item);

  // Collect sums into thread 0 and write to output.
  const unsigned long long int sum = BlockReduce(sum_storage).Sum(item);
  if (threadIdx.x == 0) {
    output[blockIdx.x] = sum;
  }
}

__global__ void part1_1024threads(const uint64_t *input, uint64_t *output) {
  using BlockReduce =
      cub::BlockReduce<unsigned long long int, 1024,
                       cub::BLOCK_REDUCE_RAKING_COMMUTATIVE_ONLY>;
  __shared__ typename BlockReduce::TempStorage sum_storage;

  const int idx = (blockDim.x * blockIdx.x) + threadIdx.x;
  uint64_t item = input[idx];

  part1_inner(&item);

  // Collect sums into thread 0 and write to output.
  unsigned long long int sum = BlockReduce(sum_storage).Sum(item);
  if (threadIdx.x == 0) {
    output[blockIdx.x] = sum;
  }
}

__device__ void part2_inner(uint64_t *item) {
  const uint32_t num_digits = cuda::ilog10(*item) + 1;

  uint64_t div = 1;

  // All these loops should match for most warps.
  // Close numbers will give the same digit counts.
  // Otherwise they will diverge on the extra outer loop.
  for (uint32_t cut = 1; cut <= (num_digits / 2); ++cut) {
    // Go to the next 10**N.
    div *= 10;

    if ((num_digits % cut) == 0) {
      uint64_t rolling = *item;
      const uint64_t basis = rolling % div;
      bool passes = true;

      for (uint8_t reps = 1; reps < (num_digits / cut); ++reps) {
        rolling /= div;
        passes &= ((rolling % div) == basis);
      }

      // Divergence here with an early return.
      // Item left untouched to be included in sum.
      if (passes) {
        return;
      }
    }
  }

  // No prior pass, set to zero.
  *item = 0;
}

__global__ void part2_512threads(const uint64_t *input, uint64_t *output) {
  using BlockReduce =
      cub::BlockReduce<unsigned long long int, 512,
                       cub::BLOCK_REDUCE_RAKING_COMMUTATIVE_ONLY>;
  __shared__ typename BlockReduce::TempStorage sum_storage;

  int idx = (blockDim.x * blockIdx.x) + threadIdx.x;
  uint64_t item = input[idx];

  part2_inner(&item);

  // Collect sums into thread 0 and write to output.
  unsigned long long int sum = BlockReduce(sum_storage).Sum(item);
  if (threadIdx.x == 0) {
    output[blockIdx.x] = sum;
  }
}

extern "C" {
struct maybeCudaStream {
  cudaError_t err;
  cudaStream_t stream;
};

struct maybeCudaBuffer {
  cudaError_t err;
  size_t buffer_size;
  uint64_t *buffer;
  cudaEvent_t event;
};

struct maybeKernelResults {
  cudaError_t err;
  uint64_t *cuda_buffer;
  cudaEvent_t event;
};

struct maybeSum {
  cudaError_t err;
  uint64_t sum;
};

maybeCudaStream init_stream(void) {
  cudaStream_t stream;
  cudaError_t err = cudaStreamCreate(&stream);

  return maybeCudaStream{err, stream};
}

/// Blocks until the stream is complete.
cudaError_t destroy_stream(cudaStream_t stream) {
  cudaError_t err = cudaStreamSynchronize(stream);
  err = (err != cudaSuccess) ? err : cudaStreamDestroy(stream);
  return err;
}

/// Returns the async cuda buffer.
/// The data must be preserved until the event completes.
maybeCudaBuffer copy_to_device(cudaStream_t stream, size_t size,
                               uint64_t *data) {
  uint64_t *buffer;
  cudaEvent_t event;

  // warpSize == 32
  // block size == 512
  size_t oversize = size % 512;
  size_t leftover = (oversize == 0) ? 0 : (512 - oversize);

  cudaError_t err = cudaEventCreate(&event);

  err = (err != cudaSuccess)
            ? err
            : cudaMallocAsync(&buffer, sizeof(uint64_t) * (size + leftover),
                              stream);
  // Copy all known values and fill the rest with 1s to fail the tests
  err = (err != cudaSuccess)
            ? err
            : cudaMemcpyAsync(buffer, data, sizeof(uint64_t) * size,
                              cudaMemcpyHostToDevice, stream);
  // Zeroes are noop in the sum
  err = (err != cudaSuccess)
            ? cudaMemsetAsync(buffer + size, 0, sizeof(uint64_t) * leftover,
                              stream)
            : err;

  err = (err != cudaSuccess) ? err : cudaEventRecord(event, stream);

  return maybeCudaBuffer{err, size + leftover, buffer, event};
}

cudaError_t destroy_buffer(cudaStream_t stream, uint64_t *buffer) {
  return cudaFreeAsync(buffer, stream);
}

cudaError_t block_and_destroy_event(cudaEvent_t event) {
  cudaError_t err = cudaEventSynchronize(event);
  err = (err != cudaSuccess) ? err : cudaEventDestroy(event);
  return err;
}

int maxThreadsPerBlock(void) {
  /*
cudaDeviceProp properties;
cudaGetDeviceProperties(&properties, 0);
return properties.maxThreadsPerBlock;
*/
  return 512;
}

int numBlocks(size_t buffer_size) {
  int maxThreads = maxThreadsPerBlock();
  return (buffer_size + (maxThreads - 1)) / maxThreads;
}

/// Blocks until the kernel is done, cleans up memory, and sums the results.
maybeSum sum(cudaStream_t stream, const uint64_t *res_cuda_buffer,
             size_t buffer_size) {
  cudaEvent_t event;
  cudaEventCreate(&event);

  int num_blocks = numBlocks(buffer_size);

  uint64_t *host_buffer = (uint64_t *)malloc(sizeof(uint64_t) * num_blocks);

  // Copy to host and set event to capture when the buffer is valid.
  cudaError_t err = cudaMemcpyAsync(host_buffer, res_cuda_buffer,
                                    sizeof(uint64_t) * num_blocks,
                                    cudaMemcpyDeviceToHost, stream);
  // Block until the host buffer has been filled.
  err = (err != cudaSuccess) ? err : cudaEventRecord(event, stream);
  err = (err != cudaSuccess) ? err : cudaEventSynchronize(event);
  err = (err != cudaSuccess) ? err : cudaEventDestroy(event);

  uint64_t sum = 0;
  for (int i = 0; i < num_blocks; ++i) {
    sum += host_buffer[i];
  }
  free(host_buffer);

  return maybeSum{err, sum};
}

maybeKernelResults run_part1(cudaStream_t stream, const maybeCudaBuffer *init) {
  uint64_t *cuda_results;
  cudaEvent_t event;
  cudaEventCreate(&event);

  int num_blocks = numBlocks(init->buffer_size);

  cudaError_t err = cudaStreamWaitEvent(stream, init->event);
  err = (err != cudaSuccess)
            ? err
            : cudaMallocAsync(&cuda_results, sizeof(uint64_t) * num_blocks,
                              stream);

  int max_threads_block = maxThreadsPerBlock();
  /*
  if (max_threads_block >= 1024) {
    part1_1024threads<<<num_blocks, 1024, 0, stream>>>(init->buffer,
                                                       cuda_results);
  } else if (max_threads_block >= 512) {
  */
  part1_512threads<<<num_blocks, 512, 0, stream>>>(init->buffer, cuda_results);
  /*
} else {
exit(1);
}
*/
  err = (err != cudaSuccess) ? err : cudaGetLastError();

  // When this event completes, the cuda results are filled.
  err = (err != cudaSuccess) ? err : cudaEventRecord(event, stream);

  return maybeKernelResults{err, cuda_results, event};
}

maybeKernelResults run_part2(cudaStream_t stream, const maybeCudaBuffer *init) {
  uint64_t *cuda_results;
  cudaEvent_t event;
  cudaEventCreate(&event);

  int num_blocks = numBlocks(init->buffer_size);

  cudaError_t err = cudaStreamWaitEvent(stream, init->event);
  err = (err != cudaSuccess)
            ? err
            : cudaMallocAsync(&cuda_results, sizeof(uint64_t) * num_blocks,
                              stream);

  int max_threads_block = maxThreadsPerBlock();
  /*
  if (max_threads_block >= 1024) {
    part2_1024threads<<<num_blocks, 1024, 0, stream>>>(init->buffer,
                                                       cuda_results);
  } else if (max_threads_block >= 512) {
  */
  part2_512threads<<<num_blocks, 512, 0, stream>>>(init->buffer, cuda_results);
  /*
} else {
exit(1);
}
*/
  err = (err != cudaSuccess) ? err : cudaGetLastError();

  // When this event completes, the cuda results are filled.
  err = (err != cudaSuccess) ? err : cudaEventRecord(event, stream);

  return maybeKernelResults{err, cuda_results, event};
}

const char *cuda_err_string(cudaError_t error) {
  return cudaGetErrorString(error);
}

cudaError_t cuda_success_value(void) { return cudaSuccess; }
}
