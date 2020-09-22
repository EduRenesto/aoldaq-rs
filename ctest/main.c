#include "../aoldaq.h"

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <unistd.h>

int main(int argc, char* argv[]) {
    uint32_t addrs[2] = { 1, 2 };
    NiFpgaArgs nifpga = {
        .bitfile = "dark matter",
        .signature = "is so",
        .resource = "divine",
        .attribute = 420,
        .addrs = &addrs[0]
    };

    const aoldaq_args_t args = {
        .n_channels = 2,
        .block_size = 4000,
        .mode = AOLDAQ_MODE_NI_FPGA,
        .nifpga = &nifpga,
    };

    aoldaq_t *instance = aoldaq_create_instance(&args);

    uint32_t session = aoldaq_get_nifpga_session(instance);
    printf("NiFpga Session: %d\n", session);

    aoldaq_start(instance);
    sleep(30);
    aoldaq_stop(instance);
    sleep(3);

    size_t n1 = aoldaq_get_data(instance, 0, 0, NULL);
    size_t n2 = aoldaq_get_data(instance, 1, 0, NULL);
    printf("Amount of data in channels: %ld, %ld\n", n1, n2);

    uint32_t *data = malloc(sizeof(uint32_t) * 42);
    size_t n_read = aoldaq_get_data(instance, 0, 42, data);

    printf("[ ");
    for(int i = 0; i < 42; i++) {
        printf("%u ", data[i]);
    }
    printf("]\n");

    n1 = aoldaq_get_data(instance, 0, 0, NULL);
    n2 = aoldaq_get_data(instance, 1, 0, NULL);
    printf("Amount of data in channels: %ld, %ld\n", n1, n2);

    n_read = aoldaq_get_data(instance, 0, 42, data);
    printf("[ ");
    for(int i = 0; i < 42; i++) {
        printf("%u ", data[i]);
    }
    printf("]\n");

    n1 = aoldaq_get_data(instance, 0, 0, NULL);
    n2 = aoldaq_get_data(instance, 1, 0, NULL);
    printf("Amount of data in channels: %ld, %ld\n", n1, n2);

    aoldaq_flush_fifo(instance, 0);
    aoldaq_flush_fifo(instance, 1);

    n1 = aoldaq_get_data(instance, 0, 0, NULL);
    n2 = aoldaq_get_data(instance, 1, 0, NULL);
    printf("Amount of data in channels: %ld, %ld\n", n1, n2);

    aoldaq_destroy_instance(instance);

    return 0;
}
