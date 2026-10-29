namespace device {
struct Buffer {
    float* values;
};
__global__ void update(float* values) {
    values[threadIdx.x] = 0;
}
}
