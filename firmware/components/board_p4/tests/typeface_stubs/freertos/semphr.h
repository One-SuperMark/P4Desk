#pragma once
#include <pthread.h>
typedef struct { pthread_mutex_t lock; } StaticSemaphore_t;
typedef StaticSemaphore_t *SemaphoreHandle_t;
SemaphoreHandle_t xSemaphoreCreateMutexStatic(StaticSemaphore_t *storage);
int xSemaphoreTake(SemaphoreHandle_t mutex, unsigned timeout);
int xSemaphoreGive(SemaphoreHandle_t mutex);
