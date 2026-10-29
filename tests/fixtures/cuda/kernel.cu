#include "../include/a.hpp"
__global__ void kernel(float* x) { x[threadIdx.x] = 1.0f; }
