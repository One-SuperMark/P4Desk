#include <fcntl.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "board_p4.h"
#include "p4desk_hal.h"
#include "p4desk_runtime.h"
#include "p4desk_radio.h"
#include "esp_check.h"
#include "esp_heap_caps.h"
#include "esp_log.h"
#include "esp_partition.h"
#include "esp_random.h"
#include "esp_spiffs.h"
#include "esp_system.h"
#include "driver/usb_serial_jtag.h"

static const char *TAG = "p4desk";
_Static_assert(ESP_RST_POWERON == 1 && ESP_RST_BROWNOUT == 9 && ESP_RST_USB == 11 &&
               ESP_RST_PWR_GLITCH == 14 && ESP_RST_CPU_LOCKUP == 15, "reset-reason HAL ABI");

uint32_t p4desk_reset_reason(void)
{
    return (uint32_t)esp_reset_reason();
}

bool p4desk_typec_host_connected(void)
{
    // Receives host SOF at the native FS Type-C port; wall chargers and the
    // separate CH343 port are unobservable. Does not touch reset/DTR/RTS.
    return usb_serial_jtag_is_connected();
}

static bool partition_is_blank(const esp_partition_t *partition)
{
    uint8_t *buffer = heap_caps_malloc(4096, MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT);
    if (!buffer) return false;
    bool blank = true;
    for (size_t offset = 0; offset < partition->size && blank; offset += 4096) {
        size_t count = partition->size - offset;
        if (count > 4096) count = 4096;
        if (esp_partition_read(partition, offset, buffer, count) != ESP_OK) {
            blank = false;
            break;
        }
        for (size_t n = 0; n < count; n++) {
            if (buffer[n] != 0xff) { blank = false; break; }
        }
    }
    free(buffer);
    return blank;
}

static bool flash_check(void)
{
    // A short independent basename fits SPIFFS's 32-byte name limit. O_EXCL
    // prevents touching an existing file, including after an interrupted boot.
    char path[40];
    int file = -1;
    for (unsigned attempt = 0; attempt < 3 && file < 0; attempt++) {
        snprintf(path, sizeof(path), "/flash/.p4check-%08" PRIx32, esp_random());
        file = open(path, O_RDWR | O_CREAT | O_EXCL, 0600);
    }
    if (file < 0) {
        ESP_LOGW(TAG, "settings storage read/write check=failed stage=create");
        return false;
    }
    static const uint8_t expected[16] = {0x50, 0x34, 0x44, 0x65, 0x73, 0x6b, 0x46, 0x53,
                                       0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef};
    uint8_t observed[sizeof(expected)] = {0};
    const char *failure = NULL;
    if (write(file, expected, sizeof(expected)) != sizeof(expected)) failure = "write";
    else if (fsync(file) != 0) failure = "fsync";
    else if (lseek(file, 0, SEEK_SET) != 0) failure = "seek";
    else if (read(file, observed, sizeof(observed)) != sizeof(observed)) failure = "read";
    else if (memcmp(expected, observed, sizeof(expected)) != 0) failure = "compare";
    if (close(file) != 0 && !failure) failure = "close";
    if (unlink(path) != 0 && !failure) failure = "cleanup";
    if (failure) {
        ESP_LOGW(TAG, "settings storage read/write check=failed stage=%s", failure);
        return false;
    }
    size_t total = 0, used = 0;
    if (esp_spiffs_info("p4settings", &total, &used) == ESP_OK && total >= used)
        ESP_LOGI(TAG, "settings storage ready at /flash (p4settings), total=%zu free=%zu read/write check=passed",
                 total, total - used);
    else ESP_LOGI(TAG, "settings storage ready at /flash (p4settings), read/write check=passed");
    return true;
}

static void flash_mount(void)
{
    const esp_vfs_spiffs_conf_t config = {
        .base_path = "/flash", .partition_label = "p4settings", .max_files = 8,
        .format_if_mount_failed = false,
    };
    esp_err_t err = esp_vfs_spiffs_register(&config);
    // Only a genuinely empty new partition may be initialized. Existing data is
    // preserved on a mount failure; neither corrupted flash nor TF is formatted.
    if (err != ESP_OK) {
        const esp_partition_t *partition = esp_partition_find_first(
            ESP_PARTITION_TYPE_DATA, ESP_PARTITION_SUBTYPE_DATA_SPIFFS, "p4settings");
        if (partition && partition_is_blank(partition)) {
            ESP_LOGI(TAG, "initializing blank settings partition (p4settings)");
            err = esp_spiffs_format("p4settings");
            if (err == ESP_OK) err = esp_vfs_spiffs_register(&config);
        }
    }
    if (err != ESP_OK) {
        ESP_LOGW(TAG, "settings storage unavailable; data preserved: %s", esp_err_to_name(err));
        return;
    }
    ESP_LOGI(TAG, "settings storage mounted at /flash (p4settings)");
    if (!flash_check()) ESP_LOGW(TAG, "settings persistence unavailable; read/write self-check failed");
}

void app_main(void)
{
    static board_p4_t board;
    ESP_LOGI(TAG, "P4Desk starting, protocol v1");
    ESP_LOGI(TAG, "boot reset_reason=%" PRIu32, p4desk_reset_reason());
    ESP_ERROR_CHECK(board_p4_init(&board));
    flash_mount();
    p4desk_runtime_init(&board);
    p4desk_usb_init();
    p4desk_radio_init();
    ESP_LOGI(TAG, "Rust UI starting");
    rust_main_entry();
    ESP_LOGE(TAG, "Rust UI returned unexpectedly");
    abort();
}
