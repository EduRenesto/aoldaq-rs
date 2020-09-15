#ifndef AOLDAQ_H
#define AOLDAQ_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum {
  AOLDAQ_MODE_NI_FPGA,
  AOLDAQ_MODE_RANDOM,
} aoldaq_mode;

typedef struct aoldaq_t aoldaq_t;

typedef struct {
  const char *bitfile;
  const char *signature;
  const char *resource;
  uint32_t attribute;
  const uint32_t *addrs;
} NiFpgaArgs;

typedef struct {
  uintptr_t n_channels;
  aoldaq_mode mode;
  const NiFpgaArgs *nifpga;
} aoldaq_args_t;

/**
 * Creates an AOLDAQ instance.
 */
aoldaq_t *aoldaq_create_instance(const aoldaq_args_t *args);

/**
 * Destroys an AOLDAQ instance, stopping the threads and dropping everything.
 */
void aoldaq_destroy_instance(aoldaq_t *instance);

/**
 * Consumes and frees everything in the specified channel.
 */
void aoldaq_flush_fifo(aoldaq_t *instance, uintptr_t channel);

/**
 * Tries to return `n` `uint32_t`s of data, returning 0 if unsuccesufl.
 * Assumes that `buf` is a preallocated buffer capable of receiving all the data.
 */
uintptr_t aoldaq_get_data(aoldaq_t *instance, uintptr_t channel, uintptr_t n, uint32_t *buf);

/**
 * Returns the underlying NiFPGA session object.
 */
uint32_t aoldaq_get_nifpga_session(aoldaq_t *instance);

/**
 * Unparks the threads and starts the acquisition.
 */
void aoldaq_start(aoldaq_t *instance);

/**
 * Parks the threads, pausing the acquisition.
 */
void aoldaq_stop(aoldaq_t *instance);

#endif /* AOLDAQ_H */
